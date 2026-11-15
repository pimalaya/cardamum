//! # Microsoft Graph backend
//!
//! The Graph arm of the unified-API client, mapping the addressbook and card
//! operations onto [`io_msgraph::v1::client::MsgraphClientStd`] calls and
//! projecting contacts onto vCard documents through io-msgraph's `vcard`
//! feature.
//!
//! Graph contact folders are the addressbooks, except the default Contacts
//! folder, which the folders endpoint does not list: it is surfaced under
//! the [`CONTACTS_FOLDER`] sentinel id. Updates carry no If-Match guard
//! server-side (last-write-wins), so passing one bails.

use anyhow::{Error, Result, bail};
use io_msgraph::v1::{
    client::{MsgraphClientStd, MsgraphClientStdConnectOptions, MsgraphClientStdError},
    rest::users::{
        contact_folders::MsgraphContactFolder,
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
    msgraph::client::is_missing,
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
    /// folder first under the [`CONTACTS_FOLDER`] sentinel id.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let mut books = vec![Addressbook {
            id: CONTACTS_FOLDER.to_string(),
            name: "Contacts".to_string(),
            description: None,
            color: None,
            default: false,
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
        let created = self.inner.contact_folder_create(&folder)?.response;

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
        self.inner.contact_folder_update(id, &folder)?;

        Ok(())
    }

    /// Deletes the contact folder identified by `id` and every contact
    /// it contains.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        if id == CONTACTS_FOLDER {
            bail!("The default Contacts folder cannot be deleted");
        }

        self.inner.contact_folder_delete(id)?;
        Ok(())
    }

    /// Lists the contacts of the folder, each projected onto a vCard
    /// document, the `take` after the first `skip`.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<Vec<Card>> {
        let params = MsgraphContactsListParams {
            top: Some(100),
            expand: Some(MSGRAPH_CONTACT_STASH_EXPAND),
            ..Default::default()
        };

        let mut current = self
            .inner
            .contacts_list(folder(addressbook_id), &params)?
            .response;

        let mut cards = Vec::new();
        loop {
            cards.extend(
                current
                    .value
                    .into_iter()
                    .map(|contact| into_card(addressbook_id, contact)),
            );

            match current.next_link {
                Some(next) => current = self.inner.page_get(&next, None)?.response,
                None => break,
            }
        }

        Ok(window(cards, skip, take))
    }

    /// Reads the contact `card_id`, projected onto a vCard document.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let contact = self
            .inner
            .contact_get(card_id, Some(MSGRAPH_CONTACT_STASH_EXPAND))
            .map_err(|err| card_error(card_id, err))?
            .response;
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
            .contact_create(folder(addressbook_id), &contact)?
            .response;

        Ok(created.id)
    }

    /// Updates the contact `card_id` from the vCard.
    ///
    /// The server contact serves as delta base, so the PATCH body shrinks
    /// to the changed fields plus nulls for the removed ones. Graph updates
    /// are last-write-wins, so passing an If-Match guard bails.
    pub fn update_card(
        &mut self,
        _addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        if if_match.is_some() {
            bail!("Microsoft Graph does not support If-Match guarded updates");
        }

        let vcard = into_vcard_text(contents)?;

        let base = self
            .inner
            .contact_get(card_id, Some(MSGRAPH_CONTACT_STASH_EXPAND))
            .map_err(|err| card_error(card_id, err))?
            .response;
        let base_vcard = base.to_vcard();

        let contact = MsgraphContact::update_from_vcard(&vcard, &base_vcard)?;
        self.inner
            .contact_update(card_id, &contact)
            .map_err(|err| card_error(card_id, err))?;

        Ok(CardUpdateOutcome::default())
    }

    /// Deletes the contact `card_id`.
    pub fn delete_card(
        &mut self,
        _addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        if if_match.is_some() {
            bail!("Microsoft Graph contacts cannot gate a delete on an ETag");
        }
        self.inner
            .contact_delete(card_id)
            .map_err(|err| card_error(card_id, err))?;
        Ok(())
    }
}

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
fn card_error(card_id: &str, err: MsgraphClientStdError) -> Error {
    match is_missing(&err) {
        true => error::card_not_found(card_id, err.into()),
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
}
