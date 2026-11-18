//! # Microsoft Graph backend
//!
//! The Graph arm of the unified-API client, mapping the addressbook and card
//! operations onto [`io_msgraph::v1::client::MsgraphClientStd`] calls and
//! projecting contacts onto vCard documents through io-msgraph's `vcard`
//! feature.
//!
//! Graph contact folders are the addressbooks, except the default Contacts
//! folder, which the folders endpoint does not list: it is surfaced under
//! the [`CONTACTS_FOLDER`] sentinel id. Graph takes no If-Match on a
//! contact write (last-write-wins), so `--if-match` is compared with the
//! `changeKey` read just before the write: the update reads the contact as
//! its delta base anyway, the delete reads it first. Each read checks the
//! contact's folder, so a read, an update or a delete acts only on a
//! contact the named folder holds.

use anyhow::{Error, Result, bail};
use io_msgraph::v1::{
    client::{MsgraphClientStd, MsgraphClientStdConnectOptions, MsgraphClientStdError},
    rest::users::{
        contact_folders::{MsgraphContactFolder, list::MsgraphContactFoldersListParams},
        contacts::{
            MsgraphContact, list::MsgraphContactsListParams, vcard::MSGRAPH_CONTACT_STASH_EXPAND,
        },
    },
};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    config::{MsgraphConfig, ProxyConfig},
    error,
    msgraph::client::{is_folder_taken, is_missing},
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
        client::window,
    },
};

/// Sentinel addressbook id of the default Contacts folder, which Graph
/// addresses by omitting the folder segment.
pub const CONTACTS_FOLDER: &str = "contacts";

/// Microsoft Graph backend of the unified-API client.
pub struct MsgraphBackend {
    pub inner: MsgraphClientStd,
}

impl MsgraphBackend {
    /// Connects to the Graph API from the account's `[msgraph]` block.
    ///
    /// The credential is resolved through `resolver`, so an account naming
    /// one credential command from several of its backends spawns it once.
    pub fn new(config: MsgraphConfig, resolver: &mut SecretResolver) -> Result<Self> {
        let token = resolver.resolve(config.auth.token)?;
        let options = MsgraphClientStdConnectOptions {
            tls: config.tls.into_tls(config.alpn),
            proxy: ProxyConfig::resolve(config.proxy, resolver)?,
            user_id: config.user_id,
        };
        let inner = MsgraphClientStd::connect(token.expose_secret(), options)?;
        Ok(Self { inner })
    }

    /// Lists the contact folders as addressbooks, the default Contacts
    /// folder first under the [`CONTACTS_FOLDER`] sentinel id, marked
    /// `default`: the folder Graph creates a contact in when none is named.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let mut books = vec![Addressbook {
            id: CONTACTS_FOLDER.to_string(),
            name: "Contacts".to_string(),
            description: None,
            color: None,
            default: true,
        }];

        let mut page = self
            .inner
            .contact_folders_list(&Default::default())?
            .response;

        loop {
            for folder in page.value {
                books.push(Addressbook {
                    name: if folder.display_name.is_empty() {
                        folder.id.clone()
                    } else {
                        folder.display_name
                    },
                    id: folder.id,
                    description: None,
                    color: None,
                    default: false,
                });
            }

            match page.next_link {
                Some(next) => page = self.inner.page_get(&next, None)?.response,
                None => break,
            }
        }

        Ok(books)
    }

    /// Creates a contact folder named `name`.
    ///
    /// Graph folders carry no description nor color, so passing either
    /// bails.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if description.is_some() || color.is_some() {
            bail!("Microsoft Graph contact folders support neither description nor color");
        }

        let folder = MsgraphContactFolder {
            display_name: name.to_string(),
            ..Default::default()
        };
        let created = match self.inner.contact_folder_create(&folder) {
            Ok(created) => created.response,
            Err(err) if is_folder_taken(&err) => {
                return Err(error::addressbook_duplicate(name, err.into()));
            }
            Err(err) => return Err(err.into()),
        };

        Ok(created.id)
    }

    /// Renames the contact folder identified by `id`.
    ///
    /// Graph folders carry no description nor color, so patching either
    /// bails.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        if patch.description.is_some() || patch.color.is_some() {
            bail!("Microsoft Graph contact folders support neither description nor color");
        }
        if id == CONTACTS_FOLDER {
            bail!("The default Contacts folder cannot be updated");
        }

        let Some(name) = patch.name else {
            return Ok(());
        };

        let folder = MsgraphContactFolder {
            display_name: name,
            ..Default::default()
        };
        self.inner
            .contact_folder_update(id, &folder)
            .map_err(|err| folder_error(id, err))?;

        Ok(())
    }

    /// Deletes the contact folder identified by `id` and every contact
    /// it contains.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        if id == CONTACTS_FOLDER {
            bail!("The default Contacts folder cannot be deleted");
        }

        self.inner
            .contact_folder_delete(id)
            .map_err(|err| folder_error(id, err))?;
        Ok(())
    }

    /// Lists the contacts of the folder, each projected onto a vCard
    /// document, the `take` after the first `skip`.
    ///
    /// Graph is asked for that window alone (`$skip`, `$top` capped at
    /// [`PAGE_SIZE`]), and a next link is followed only until the window
    /// is held, Graph answering fewer contacts than asked when it pleases.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<Vec<Card>> {
        let top = take.map_or(PAGE_SIZE, |take| take.min(PAGE_SIZE as usize) as u32);
        let params = MsgraphContactsListParams {
            top: Some(top),
            skip: (skip > 0).then(|| u32::try_from(skip).unwrap_or(u32::MAX)),
            expand: Some(MSGRAPH_CONTACT_STASH_EXPAND),
            ..Default::default()
        };

        let mut current = self
            .inner
            .contacts_list(folder(addressbook_id), &params)
            .map_err(|err| folder_error(addressbook_id, err))?
            .response;

        let mut cards = Vec::new();
        loop {
            cards.extend(
                current
                    .value
                    .into_iter()
                    .map(|contact| into_card(addressbook_id, contact)),
            );

            if take.is_some_and(|take| cards.len() >= take) {
                break;
            }

            match current.next_link {
                Some(next) => current = self.inner.page_get(&next, None)?.response,
                None => break,
            }
        }

        Ok(window(cards, 0, take))
    }

    /// Reads the contact `card_id`, `$expand`ing `expand` when given,
    /// failing `card-not-found` unless the folder `addressbook_id` holds it.
    ///
    /// Graph reads a contact by id whatever its folder, so its
    /// `parentFolderId` is compared with the folder's id. The default
    /// Contacts folder has no id under its sentinel: it is the parent of
    /// the folders Graph lists, one of which is read, and holds every
    /// contact when there is none.
    fn held_contact(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        expand: Option<&str>,
    ) -> Result<MsgraphContact> {
        let contact = self
            .inner
            .contact_get(card_id, expand)
            .map_err(|err| card_error(addressbook_id, card_id, err))?
            .response;
        let parent = contact.parent_folder_id.as_deref();

        let held = match folder(addressbook_id) {
            Some(id) => parent == Some(id),
            None => {
                let params = MsgraphContactFoldersListParams {
                    top: Some(1),
                    select: Some("parentFolderId"),
                    ..Default::default()
                };
                let folders = self.inner.contact_folders_list(&params)?.response;

                match folders.value.first() {
                    Some(folder) => {
                        parent.is_some() && parent == folder.parent_folder_id.as_deref()
                    }
                    None => true,
                }
            }
        };

        if !held {
            let cause = anyhow::anyhow!("Graph files the contact in another folder");
            return Err(error::card_not_found(addressbook_id, card_id, cause));
        }

        Ok(contact)
    }

    /// Reads the contact `card_id`, projected onto a vCard document.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let contact =
            self.held_contact(addressbook_id, card_id, Some(MSGRAPH_CONTACT_STASH_EXPAND))?;
        Ok(into_card(addressbook_id, contact))
    }

    /// Creates the vCard as a Graph contact in the folder.
    ///
    /// Graph names the resource, so the returned id is server-assigned.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let vcard = into_vcard_text(contents)?;
        let contact = MsgraphContact::create_from_vcard(&vcard)?;

        let created = self
            .inner
            .contact_create(folder(addressbook_id), &contact)
            .map_err(|err| folder_error(addressbook_id, err))?
            .response;

        Ok(created.id)
    }

    /// Updates the contact `card_id` from the vCard.
    ///
    /// The server contact serves as delta base, so the PATCH body shrinks
    /// to the changed fields plus nulls for the removed ones. Graph updates
    /// are last-write-wins, so `if_match` is compared with the base's
    /// `changeKey`, a write landing between that read and the PATCH not
    /// caught.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        let vcard = into_vcard_text(contents)?;

        let base =
            self.held_contact(addressbook_id, card_id, Some(MSGRAPH_CONTACT_STASH_EXPAND))?;

        if let Some(expected) = if_match
            && !error::version_matches(base.change_key.as_deref(), expected)
        {
            return Err(error::card_changed(card_id, SERVER));
        }

        let base_vcard = base.to_vcard();

        let contact = MsgraphContact::update_from_vcard(&vcard, &base_vcard)?;
        self.inner
            .contact_update(card_id, &contact)
            .map_err(|err| card_error(addressbook_id, card_id, err))?;

        Ok(CardUpdateOutcome::default())
    }

    /// Deletes the contact `card_id`.
    ///
    /// The contact is read first, so one the folder `addressbook_id` does
    /// not hold is a card not found there, nothing deleted, and an
    /// `if_match` is compared with its `changeKey`, as an update compares
    /// it.
    pub fn delete_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        let current = self.held_contact(addressbook_id, card_id, None)?;

        if let Some(expected) = if_match
            && !error::version_matches(current.change_key.as_deref(), expected)
        {
            return Err(error::card_changed(card_id, SERVER));
        }

        self.inner
            .contact_delete(card_id)
            .map_err(|err| card_error(addressbook_id, card_id, err))?;
        Ok(())
    }
}

/// The server a refused guard names.
const SERVER: &str = "Microsoft Graph";

/// The most contacts one listing request asks Graph for (`$top`).
const PAGE_SIZE: u32 = 100;

/// Maps the sentinel Contacts folder id to the omitted folder segment.
fn folder(addressbook_id: &str) -> Option<&str> {
    (addressbook_id != CONTACTS_FOLDER).then_some(addressbook_id)
}

/// io-msgraph contact to the shared card shape: the projected vCard
/// document as contents, the Graph id as id and the changeKey as ETag.
fn into_card(addressbook_id: &str, contact: MsgraphContact) -> Card {
    let vcard = contact.to_vcard();
    Card {
        id: contact.id,
        addressbook_id: addressbook_id.to_string(),
        etag: contact.change_key,
        contents: vcard.into_bytes(),
    }
}

/// Decodes raw card bytes as UTF-8 vCard text.
fn into_vcard_text(contents: Vec<u8>) -> Result<String> {
    String::from_utf8(contents).map_err(|_| anyhow::anyhow!("Card contents are not valid UTF-8"))
}

/// The failure of a request on the contact `card_id`, `card-not-found`
/// when Graph holds no such contact.
fn card_error(addressbook_id: &str, card_id: &str, err: MsgraphClientStdError) -> Error {
    match is_missing(&err) {
        true => error::card_not_found(addressbook_id, card_id, err.into()),
        false => err.into(),
    }
}

/// The failure of a request on the contact folder `id`,
/// `addressbook-not-found` naming it when Graph holds no such folder or
/// the id cannot name one, Graph's answer in `sources`.
fn folder_error(id: &str, err: MsgraphClientStdError) -> Error {
    match is_missing(&err) {
        true => error::addressbook_not_found(id, err.into()),
        false => err.into(),
    }
}

#[cfg(test)]
mod tests {
    use io_msgraph::v1::client::MsgraphClientStd;
    use pimalaya_cli::error::{code_of, with_code};
    use serde_json::json;

    use super::{CONTACTS_FOLDER, MsgraphBackend};
    use crate::{
        error::{self, ErrorCode},
        script::{self, Script},
    };

    /// The backend over a Graph server answering `status` with an error
    /// envelope of `code`.
    fn refusing(status: u16, code: &str) -> MsgraphBackend {
        let body = json!({"error": {"code": code, "message": "Refused"}}).to_string();
        let (script, _) = Script::new([script::response(status, "application/json", &body)]);
        MsgraphBackend {
            inner: MsgraphClientStd::new(script, "token", Default::default()),
        }
    }

    /// The code a delete of a contact refused with `status` and `code`
    /// prints in its failure object.
    fn delete_code(status: u16, code: &str) -> Option<ErrorCode> {
        let err = refusing(status, code)
            .delete_card(CONTACTS_FOLDER, "AAMk", None)
            .unwrap_err();
        let code = error::code(&err);
        assert_eq!(code_of(&with_code(err, error::code)), code);
        code
    }

    #[test]
    fn a_graph_refusal_carries_its_code() {
        let denied = Some(ErrorCode::PermissionDenied);
        let quota = Some(ErrorCode::QuotaExceeded);

        assert_eq!(delete_code(403, "ErrorAccessDenied"), denied);
        assert_eq!(delete_code(507, "ErrorQuotaExceeded"), quota);
        assert_eq!(delete_code(400, "ErrorQuotaExceeded"), quota);
        assert_eq!(
            delete_code(429, "TooManyRequests"),
            Some(ErrorCode::RateLimited)
        );
        assert_eq!(delete_code(500, "ErrorInternalServerError"), None);
    }

    /// A folder Graph refuses as one of that name existing is
    /// `addressbook-duplicate` naming it, Graph's answer in `sources`.
    #[test]
    fn an_address_book_created_twice_is_a_duplicate() {
        let err = refusing(409, "ErrorFolderExists")
            .create_addressbook("Work", None, None)
            .unwrap_err();
        let duplicate = Some(ErrorCode::AddressbookDuplicate);
        assert_eq!(code_of(&err), duplicate, "{err:#}");
        assert_eq!(
            err.to_string(),
            "Cannot create address book `Work`: it exists already"
        );
        assert!(format!("{err:#}").contains("Refused"), "{err:#}");

        let err = refusing(403, "ErrorAccessDenied")
            .create_addressbook("Work", None, None)
            .unwrap_err();
        assert_eq!(error::code(&err), Some(ErrorCode::PermissionDenied));
    }

    /// The backend over a Graph server answering each request with the
    /// next of `responses`, and the requests it receives.
    fn scripted(responses: Vec<Vec<u8>>) -> (MsgraphBackend, script::Requests) {
        let (script, requests) = Script::new(responses);
        let inner = MsgraphClientStd::new(script, "token", Default::default());
        (MsgraphBackend { inner }, requests)
    }

    /// The methods of the requests a server received, in order.
    fn methods(requests: &script::Requests) -> Vec<String> {
        let requests = requests.lock().unwrap();
        requests
            .iter()
            .map(|request| {
                let request = String::from_utf8_lossy(request);
                request.split(' ').next().unwrap_or_default().to_owned()
            })
            .collect()
    }

    /// The user folder the guard tests address, holding the contact `AAMk`.
    const FOLDER: &str = "AAMkF";

    const CARD: &[u8] = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada Byron\r\nEND:VCARD\r\n";

    /// An update `--if-match` guards is compared with the `changeKey` of
    /// the contact it reads as its base anyway: a stale one is refused
    /// `precondition-failed` before the `PATCH`, the one read lets it
    /// through, quoted or not, no request added.
    #[test]
    fn an_update_is_guarded_by_the_change_key_it_reads() {
        let (mut backend, requests) = scripted(vec![contact_in(FOLDER)]);
        let err = backend
            .update_card(FOLDER, "AAMk", CARD.to_vec(), Some("k0"))
            .unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::PreconditionFailed));
        assert_eq!(methods(&requests), ["GET"]);

        let patched = script::response(200, "application/json", "{}");
        let (mut backend, requests) = scripted(vec![contact_in(FOLDER), patched]);
        backend
            .update_card(FOLDER, "AAMk", CARD.to_vec(), Some("\"k1\""))
            .unwrap();
        assert_eq!(methods(&requests), ["GET", "PATCH"]);
    }

    /// A delete reads the contact first, refusing a stale `changeKey` an
    /// `--if-match` names before the `DELETE`.
    #[test]
    fn a_delete_is_guarded_by_the_change_key_it_reads_first() {
        let (mut backend, requests) = scripted(vec![contact_in(FOLDER)]);
        let err = backend.delete_card(FOLDER, "AAMk", Some("k0")).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::PreconditionFailed));
        assert!(
            err.to_string().contains("changed on Microsoft Graph"),
            "{err}"
        );
        assert_eq!(methods(&requests), ["GET"]);

        let deleted = script::response(204, "application/json", "");
        let (mut backend, requests) = scripted(vec![contact_in(FOLDER), deleted.clone()]);
        backend.delete_card(FOLDER, "AAMk", Some("k1")).unwrap();
        assert_eq!(methods(&requests), ["GET", "DELETE"]);

        let (mut backend, requests) = scripted(vec![contact_in(FOLDER), deleted]);
        backend.delete_card(FOLDER, "AAMk", None).unwrap();
        assert_eq!(methods(&requests), ["GET", "DELETE"]);
    }

    /// The contact `AAMk` filed in the folder `parent`.
    fn contact_in(parent: &str) -> Vec<u8> {
        let body = format!(
            r#"{{"id":"AAMk","changeKey":"k1","displayName":"Ada","parentFolderId":"{parent}"}}"#
        );
        script::response(200, "application/json", &body)
    }

    /// A contact Graph holds outside the named folder is a card not found
    /// there, whether that folder is a user folder or the default Contacts
    /// one: a read does not print it under that folder, an update and a
    /// delete send no `PATCH` nor `DELETE`.
    #[test]
    fn a_contact_of_another_folder_is_not_found() {
        let missing = Some(ErrorCode::CardNotFound);
        let folders = r#"{"value":[{"id":"AAMkF","parentFolderId":"AAMkC"}]}"#;
        let folders = script::response(200, "application/json", folders);
        let done = script::response(204, "application/json", "");

        for (book, parent) in [("AAMkF", "AAMkG"), (CONTACTS_FOLDER, "AAMkF")] {
            let answers = || vec![contact_in(parent), folders.clone(), done.clone()];

            let (mut backend, _) = scripted(answers());
            let err = backend.get_card(book, "AAMk").unwrap_err();
            assert_eq!(code_of(&err), missing, "{book}: {err:#}");
            assert_eq!(
                err.to_string(),
                format!("Card `AAMk` not found in address book `{book}`")
            );

            let (mut backend, requests) = scripted(answers());
            let err = backend
                .update_card(book, "AAMk", CARD.to_vec(), None)
                .unwrap_err();
            assert_eq!(code_of(&err), missing, "{book}: {err:#}");
            assert!(!methods(&requests).contains(&"PATCH".to_owned()), "{book}");

            let (mut backend, requests) = scripted(answers());
            let err = backend.delete_card(book, "AAMk", None).unwrap_err();
            assert_eq!(code_of(&err), missing, "{book}: {err:#}");
            assert!(!methods(&requests).contains(&"DELETE".to_owned()), "{book}");
        }

        let (mut backend, requests) = scripted(vec![contact_in("AAMkC"), folders.clone()]);
        let card = backend.get_card(CONTACTS_FOLDER, "AAMk").unwrap();
        assert_eq!(card.addressbook_id, CONTACTS_FOLDER);
        assert_eq!(methods(&requests), ["GET", "GET"]);

        let none = script::response(200, "application/json", r#"{"value":[]}"#);
        let (mut backend, _) = scripted(vec![contact_in("AAMkC"), none]);
        backend.get_card(CONTACTS_FOLDER, "AAMk").unwrap();

        let (mut backend, requests) = scripted(vec![contact_in("AAMkF")]);
        backend.get_card("AAMkF", "AAMk").unwrap();
        assert_eq!(methods(&requests), ["GET"]);
    }

    /// A contact Graph does not hold, or an id that cannot name one, is a
    /// card not found; another 400 is not.
    #[test]
    fn a_missing_contact_is_a_card_not_found() {
        let missing = Some(ErrorCode::CardNotFound);

        assert_eq!(delete_code(404, "ErrorItemNotFound"), missing);
        assert_eq!(delete_code(400, "ErrorInvalidIdMalformed"), missing);
        assert_eq!(delete_code(400, "ErrorInvalidRequest"), None);

        let err = refusing(404, "ErrorItemNotFound")
            .get_card(CONTACTS_FOLDER, "AAMk")
            .unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
    }

    /// A folder Graph does not hold, or an id that cannot name one, fails
    /// `addressbook-not-found` naming it, Graph's answer in `sources`, on a
    /// listing, a card added to it and a folder write.
    #[test]
    fn an_unknown_folder_is_not_found() {
        let missing = Some(ErrorCode::AddressbookNotFound);

        let err = refusing(400, "ErrorInvalidIdMalformed")
            .list_cards("nope", 0, None)
            .unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
        assert_eq!(err.to_string(), "Addressbook `nope` not found");
        assert!(format!("{err:#}").contains("Refused"), "{err:#}");

        let card = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada\r\nEND:VCARD\r\n".to_vec();
        let err = refusing(404, "ErrorItemNotFound")
            .create_card("AAMkF", card)
            .unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");

        let err = refusing(404, "ErrorItemNotFound")
            .delete_addressbook("AAMkF")
            .unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
    }

    /// The default Contacts folder, where Graph adds a contact naming no
    /// folder, is the default address book; the other folders are not.
    #[test]
    fn the_contacts_folder_is_the_default() {
        let folders = r#"{"value":[{"id":"AAMkF","displayName":"Friends"}]}"#;
        let folders = script::response(200, "application/json", folders);
        let (mut backend, _) = scripted(vec![folders]);
        let books = backend.list_addressbooks().unwrap();

        let defaults: Vec<_> = books.iter().map(|book| (&*book.id, book.default)).collect();
        assert_eq!(defaults, [(CONTACTS_FOLDER, true), ("AAMkF", false)]);
    }

    /// A page of Graph contacts `first..first + len`, its `@odata.nextLink`
    /// pointing past it while `more`.
    fn contact_page(first: usize, len: usize, more: bool) -> Vec<u8> {
        let value: Vec<_> = (first..first + len)
            .map(|n| json!({"id": format!("AAMk{n}"), "displayName": format!("C{n}")}))
            .collect();
        let mut body = json!({"value": value});
        if more {
            let next = format!(
                "https://graph.microsoft.com/v1.0/me/contacts?$top=2&$skip={}",
                first + len
            );
            body["@odata.nextLink"] = json!(next);
        }
        script::response(200, "application/json", &body.to_string())
    }

    /// The request lines a server received, in order.
    fn request_lines(requests: &script::Requests) -> Vec<String> {
        let requests = requests.lock().unwrap();
        requests
            .iter()
            .map(|request| {
                let request = String::from_utf8_lossy(request);
                request.lines().next().unwrap_or_default().to_owned()
            })
            .collect()
    }

    /// A page of a long folder asks Graph for that page alone, `$skip`
    /// past the cards before it and `$top` the page and the card telling
    /// whether another follows, and follows a short answer's next link
    /// only until it holds them.
    #[test]
    fn a_page_reads_only_the_pages_it_needs() {
        let pages = (0..5).map(|n| contact_page(n * 2, 2, n < 4)).collect();
        let (mut backend, requests) = scripted(pages);
        let cards = backend.list_cards(CONTACTS_FOLDER, 0, Some(3)).unwrap();

        let ids: Vec<&str> = cards.iter().map(|card| card.id.as_str()).collect();
        assert_eq!(ids, ["AAMk0", "AAMk1", "AAMk2"]);
        let lines = request_lines(&requests);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(lines[0].contains("%24top=3"), "{lines:?}");
        assert!(!lines[0].contains("%24skip"), "{lines:?}");

        let (mut backend, requests) = scripted(vec![contact_page(4, 3, true)]);
        let cards = backend.list_cards(CONTACTS_FOLDER, 4, Some(3)).unwrap();
        assert_eq!(cards.len(), 3);
        let lines = request_lines(&requests);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].contains("%24skip=4"), "{lines:?}");
    }
}
