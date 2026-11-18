//! # JMAP backend
//!
//! The JMAP arm of the unified-API client: maps the unified addressbook
//! and card operations onto io-jmap calls (RFC 9610), converting
//! ContactCards to vCard documents through [`project`].
//!
//! JMAP AddressBooks are the addressbooks; ContactCards are
//! account-level with m:n memberships, so each listing queries the
//! requested AddressBook. A ContactCard write takes no If-Match
//! equivalent (last-write-wins), so `--if-match` is compared with the hash
//! of the card read just before the write: the update reads the card as
//! its patch base anyway, the delete reads it first. Each read checks the
//! card's `addressBookIds`, so a read, an update or a delete acts only on a
//! card the named AddressBook holds.

use std::collections::BTreeMap;

use anyhow::{Error, Result, anyhow, bail};
use base64_codec::codec::Base64Codec;
use io_jmap::{
    client::{JmapClientStd, JmapClientStdConnectOptions},
    rfc9610::{
        address_book::{
            JmapAddressBook,
            get::JmapAddressBookGetOptions,
            set::{JmapAddressBookCreate, JmapAddressBookSetArgs, JmapAddressBookUpdate},
        },
        contact_card::{
            JmapContactCard,
            get::JmapContactCardGetOptions,
            query::{JmapContactCardFilter, JmapContactCardQueryOptions},
            set::{JmapContactCardPatch, JmapContactCardSetArgs, JmapContactCardSetItemError},
        },
    },
};
use pimalaya_config::secret::SecretResolver;
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;

use crate::{
    config::{JmapAuthConfig, JmapConfig, ProxyConfig, parse_server},
    error::{self, ErrorCode},
    jmap::{
        error::{JmapSetError, format_set_error, set_error, set_error_message},
        project,
    },
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
        client::{CardWindow, first_page, window},
    },
};

/// The unified-API backend, over an io-jmap client with a live session.
pub struct JmapBackend {
    pub inner: JmapClientStd,
}

impl JmapBackend {
    /// Establishes the JMAP session from the account's `[jmap]` block.
    ///
    /// The credential is resolved through `resolver`, so an account naming
    /// one credential command from several of its backends spawns it once.
    pub fn new(config: JmapConfig, resolver: &mut SecretResolver) -> Result<Self> {
        let http_auth = jmap_http_auth(config.auth, resolver)?;
        let url = parse_server(&config.server, "https", &["http", "https", "jmap", "jmaps"])?;
        let options = JmapClientStdConnectOptions {
            tls: config.tls.into_tls(config.alpn),
            proxy: ProxyConfig::resolve(config.proxy, resolver)?,
        };

        let mut inner = JmapClientStd::connect(&url, http_auth, options)?;
        inner.session_get(&url)?;

        Ok(Self { inner })
    }

    /// Lists the account's AddressBooks (RFC 9610 §2.1).
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let out = self
            .inner
            .address_book_get(JmapAddressBookGetOptions::default())?;

        Ok(out
            .address_books
            .into_iter()
            .map(into_addressbook)
            .collect())
    }

    /// Creates an AddressBook and returns its server-assigned id.
    ///
    /// JMAP AddressBooks carry no color, so passing one bails.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if color.is_some() {
            bail!("JMAP AddressBooks do not support color");
        }

        let create = BTreeMap::from([(
            "c0".to_string(),
            JmapAddressBookCreate {
                name: Some(name.to_string()),
                description: description.map(str::to_string),
                ..Default::default()
            },
        )]);
        let args = JmapAddressBookSetArgs {
            create: Some(create),
            ..Default::default()
        };
        let out = self.inner.address_book_set(args)?;

        if let Some(err) = out.not_created.into_values().next() {
            return Err(set_error("JMAP AddressBook create rejected", &err));
        }

        out.created
            .into_values()
            .next()
            .and_then(|created| created.id)
            .ok_or_else(|| anyhow!("JMAP create response is missing the AddressBook id"))
    }

    /// Applies `patch` to the AddressBook identified by `id`.
    ///
    /// JMAP AddressBooks carry no color, so patching one bails.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        if patch.color.is_some() {
            bail!("JMAP AddressBooks do not support color");
        }

        let update = BTreeMap::from([(
            id.to_string(),
            JmapAddressBookUpdate {
                name: patch.name,
                description: patch.description,
                ..Default::default()
            },
        )]);
        let args = JmapAddressBookSetArgs {
            update: Some(update),
            ..Default::default()
        };
        let out = self.inner.address_book_set(args)?;

        if let Some(err) = out.not_updated.into_values().next() {
            return Err(set_error("JMAP AddressBook update rejected", &err));
        }

        Ok(())
    }

    /// Destroys the AddressBook with its exclusive cards (RFC 9610 §2.3).
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        let args = JmapAddressBookSetArgs {
            destroy: Some(vec![id.to_string()]),
            on_destroy_remove_contents: Some(true),
            ..Default::default()
        };
        let out = self.inner.address_book_set(args)?;

        if let Some(err) = out.not_destroyed.into_values().next() {
            return Err(set_error("JMAP AddressBook destroy rejected", &err));
        }

        Ok(())
    }

    /// Lists the AddressBook's cards as vCard documents, the `take` after
    /// the first `skip`.
    ///
    /// A card that does not convert to a vCard is left out, an entry naming
    /// it instead, on the first page only: the query reads every card.
    ///
    /// A query that fails or finds nothing is checked against the address
    /// books, one `AddressBook/get`, so an unknown one fails
    /// `addressbook-not-found` naming it rather than reading as empty or as
    /// the server's refusal of the filter.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<CardWindow> {
        let opts = JmapContactCardQueryOptions {
            filter: Some(JmapContactCardFilter {
                in_address_book: Some(addressbook_id.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let cards = match self.inner.contact_card_query(opts) {
            Ok(out) if !out.cards.is_empty() => out.cards,
            queried => match (self.holds_addressbook(addressbook_id)?, queried) {
                (true, queried) => queried?.cards,
                (false, Ok(_)) => {
                    let cause = anyhow!("JMAP AddressBook/get answered the id as `notFound`");
                    return Err(error::addressbook_not_found(addressbook_id, cause));
                }
                (false, Err(err)) => {
                    return Err(error::addressbook_not_found(addressbook_id, err.into()));
                }
            },
        };

        Ok(project_cards(addressbook_id, cards, skip, take))
    }

    /// Whether the account holds the AddressBook `id`.
    fn holds_addressbook(&mut self, id: &str) -> Result<bool> {
        let opts = JmapAddressBookGetOptions {
            ids: Some(vec![id.to_string()]),
            ..Default::default()
        };
        let out = self.inner.address_book_get(opts)?;

        Ok(out
            .address_books
            .iter()
            .any(|book| book.id.as_deref() == Some(id)))
    }

    /// Reads the ContactCard `card_id`, converted to a vCard document.
    ///
    /// A card the AddressBook `addressbook_id` does not hold, per its
    /// `addressBookIds`, is a card not found there.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let card = self.held_card(addressbook_id, card_id, None)?;
        project::to_card(addressbook_id, card).map_err(Error::msg)
    }

    /// Reads the ContactCard `card_id`, its `properties` alone when given,
    /// failing `card-not-found` unless the AddressBook `addressbook_id`
    /// holds it per its `addressBookIds`.
    fn held_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        properties: Option<Vec<String>>,
    ) -> Result<JmapContactCard> {
        let opts = JmapContactCardGetOptions {
            ids: Some(vec![card_id.to_string()]),
            properties,
        };
        let out = self.inner.contact_card_get(opts)?;

        let card = out.cards.into_iter().next().ok_or_else(|| {
            let cause = anyhow!("JMAP ContactCard/get answered the id as `notFound`");
            error::card_not_found(addressbook_id, card_id, cause)
        })?;

        if card.address_book_ids.get(addressbook_id) != Some(&true) {
            let cause = anyhow!("JMAP ContactCard is not in the AddressBook");
            return Err(error::card_not_found(addressbook_id, card_id, cause));
        }

        Ok(card)
    }

    /// Creates the vCard as a ContactCard, returning the id the server minted.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let vcard = into_vcard_text(contents)?;
        let card = project::to_jscontact(&vcard).map_err(Error::msg)?;
        let unmapped = unmapped_properties(card.get("vCardProps"));

        let create = BTreeMap::from([(
            "c0".to_string(),
            JmapContactCard {
                id: None,
                address_book_ids: BTreeMap::from([(addressbook_id.to_string(), true)]),
                card,
            },
        )]);
        let args = JmapContactCardSetArgs {
            create: Some(create),
            ..Default::default()
        };
        let out = self.inner.contact_card_set(args)?;

        if let Some(err) = out.not_created.into_values().next() {
            return Err(card_write_error("create", &err, &unmapped));
        }

        out.created
            .into_values()
            .next()
            .and_then(|created| created.id)
            .ok_or_else(|| anyhow!("JMAP create response is missing the card id"))
    }

    /// Updates the ContactCard `card_id` from the vCard.
    ///
    /// The current server card serves as patch base, so the patch
    /// shrinks to the changed properties, plus nulls for the removed
    /// ones. JMAP has no If-Match guard (updates are last-write-wins),
    /// so `if_match` is compared with the base's hash, a write landing
    /// between that read and the set not caught.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        let vcard = into_vcard_text(contents)?;
        let base = self.get_card(addressbook_id, card_id)?;

        if let Some(expected) = if_match
            && !error::version_matches(base.etag.as_deref(), expected)
        {
            return Err(error::card_changed(card_id, SERVER));
        }

        let base_vcard = into_vcard_text(base.contents)?;

        let patch = project::to_patch(&vcard, Some(&base_vcard)).map_err(Error::msg)?;
        let unmapped = unmapped_properties(patch.get("vCardProps"));

        let update = BTreeMap::from([(card_id.to_string(), JmapContactCardPatch(patch))]);
        let args = JmapContactCardSetArgs {
            update: Some(update),
            ..Default::default()
        };
        let out = self.inner.contact_card_set(args)?;

        if let Some(err) = out.not_updated.into_values().next() {
            let refusal = card_write_error("update", &err, &unmapped);
            return Err(card_refusal(addressbook_id, card_id, &err, refusal));
        }

        Ok(CardUpdateOutcome::default())
    }

    /// Destroys the ContactCard `card_id`.
    ///
    /// The card is read first, its `addressBookIds` alone without an
    /// `if_match`, so one the AddressBook `addressbook_id` does not hold is
    /// a card not found there, nothing destroyed. An `if_match` reads it
    /// whole, compared with its hash as an update compares it.
    pub fn delete_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        match if_match {
            Some(expected) => {
                let current = self.get_card(addressbook_id, card_id)?;

                if !error::version_matches(current.etag.as_deref(), expected) {
                    return Err(error::card_changed(card_id, SERVER));
                }
            }
            None => {
                let properties = vec!["addressBookIds".to_string()];
                self.held_card(addressbook_id, card_id, Some(properties))?;
            }
        }

        let args = JmapContactCardSetArgs {
            destroy: Some(vec![card_id.to_string()]),
            ..Default::default()
        };
        let out = self.inner.contact_card_set(args)?;

        if let Some(err) = out.not_destroyed.into_values().next() {
            let refusal = set_error("JMAP ContactCard destroy rejected", &err);
            return Err(card_refusal(addressbook_id, card_id, &err, refusal));
        }

        Ok(())
    }
}

/// The refusal of a write on the card `card_id`, worded `refusal`: a
/// `notFound` is a card not found, the server's answer in `sources`.
fn card_refusal(
    addressbook_id: &str,
    card_id: &str,
    err: &JmapContactCardSetItemError,
    refusal: Error,
) -> Error {
    match err.code() {
        Some(ErrorCode::CardNotFound) => error::card_not_found(addressbook_id, card_id, refusal),
        _ => refusal,
    }
}

/// The server a refused guard names.
const SERVER: &str = "the JMAP server";

/// Renders a [`JmapAuthConfig`] as the `Authorization` header value.
///
/// The credential goes through `resolver`, so a command it already
/// spawned for this account is not run again.
pub fn jmap_http_auth(
    config: JmapAuthConfig,
    resolver: &mut SecretResolver,
) -> Result<SecretString> {
    match config {
        JmapAuthConfig::Header(header) => Ok(resolver.resolve(header)?),
        JmapAuthConfig::Bearer { token } => {
            let token = resolver.resolve(token)?;
            Ok(format!("Bearer {}", token.expose_secret()).into())
        }
        JmapAuthConfig::Basic { username, password } => {
            let password = resolver.resolve(password)?;
            let creds = format!("{}:{}", username, password.expose_secret());
            let mut encoded = Vec::new();
            Base64Codec::default().encode_vec(creds.as_bytes(), &mut encoded);
            // NOTE: the base64 alphabet is ASCII, so this never falls back.
            let encoded = String::from_utf8(encoded).unwrap_or_default();
            Ok(format!("Basic {encoded}").into())
        }
    }
}

/// The `take` cards of `cards` after the first `skip`, as vCard documents
/// of `addressbook_id`, beside an entry for each card that does not
/// convert, on the first page only.
fn project_cards(
    addressbook_id: &str,
    cards: Vec<JmapContactCard>,
    skip: usize,
    take: Option<usize>,
) -> CardWindow {
    let mut listed = Vec::new();
    let mut errors = Vec::new();

    for card in cards {
        let id = card.id.clone();

        match project::to_card(addressbook_id, card) {
            Ok(card) => listed.push(card),
            Err(err) => {
                let err =
                    Error::msg(err).context("Cannot convert the JMAP contact card to a vCard");
                errors.push(error::unreadable_card(id.as_deref(), err));
            }
        }
    }

    CardWindow {
        cards: window(listed, skip, take),
        errors: first_page(errors, skip),
    }
}

/// Maps a [`JmapAddressBook`] to the shared shape, id as name fallback.
fn into_addressbook(book: JmapAddressBook) -> Addressbook {
    let id = book.id.unwrap_or_default();
    let name = book.name.filter(|name| !name.is_empty());

    Addressbook {
        name: name.unwrap_or_else(|| id.clone()),
        id,
        description: book.description,
        color: None,
        default: book.is_default,
    }
}

/// Decodes raw card bytes as UTF-8 vCard text.
fn into_vcard_text(contents: Vec<u8>) -> Result<String> {
    String::from_utf8(contents).map_err(|_| anyhow::anyhow!("Card contents are not valid UTF-8"))
}

/// The vCard property names carried by a JSContact `vCardProps` member.
///
/// That member holds each unmappable property in jCard syntax (RFC 9555
/// §2.15), the name being the first element of its array.
fn unmapped_properties(vcard_props: Option<&Value>) -> Vec<String> {
    let Some(Value::Array(props)) = vcard_props else {
        return Vec::new();
    };

    props
        .iter()
        .filter_map(|prop| prop.get(0)?.as_str())
        .map(|name| name.to_uppercase())
        .collect()
}

/// A rejected card write, rendered as prose and coded by its type.
///
/// A vCard property JSContact cannot model travels in the standard
/// `vCardProps` member, which some servers (Fastmail) refuse outright,
/// taking the whole write down with it. The server error only points at
/// `vCardProps`, so the hint names the properties that landed there:
/// they are what has to go for the write to pass.
fn card_write_error(verb: &str, err: &JmapContactCardSetItemError, unmapped: &[String]) -> Error {
    let msg = format!("JMAP ContactCard {verb} rejected{}", format_set_error(err));

    if !err.properties().iter().any(|prop| prop == "vCardProps") {
        return set_error_message(msg, err);
    }

    let hint = "This server refuses the standard `vCardProps` member, where a vCard property JSContact cannot model is preserved";
    let msg = match unmapped {
        [] => format!("{msg}. {hint}"),
        names => format!(
            "{msg}. {hint}; the card carries {}, which JMAP has no home for on this server",
            names.join(", ")
        ),
    };

    set_error_message(msg, err)
}

#[cfg(test)]
mod tests {
    use pimalaya_config::secret::{Secret, SecretResolver};
    use secrecy::{ExposeSecret, SecretString};

    use super::jmap_http_auth;
    use crate::config::JmapAuthConfig;

    /// The backend over a JMAP server answering each request with the next
    /// of `bodies`, and the requests it receives.
    fn scripted(bodies: &[&str]) -> (super::JmapBackend, crate::script::Requests) {
        use io_jmap::{client::JmapClientStd, rfc8620::session::JmapSession};

        let session: JmapSession = serde_json::from_value(serde_json::json!({
            "username": "", "accounts": {}, "capabilities": {},
            "primaryAccounts": {"urn:ietf:params:jmap:contacts": "a1"},
            "apiUrl": "https://jmap.example.org/api/", "downloadUrl": "",
            "uploadUrl": "", "eventSourceUrl": "", "state": "",
        }))
        .unwrap();
        let responses = bodies
            .iter()
            .map(|body| crate::script::response(200, "application/json", body));
        let (script, requests) = crate::script::Script::new(responses);
        let auth = SecretString::from("Bearer token");
        let inner = JmapClientStd::from_parts(script, auth, session);

        (super::JmapBackend { inner }, requests)
    }

    /// A `ContactCard/get` answering the card `c1`.
    const GET: &str = r#"{"methodResponses": [["ContactCard/get", {
        "accountId": "a1", "state": "s1", "notFound": [],
        "list": [{"id": "c1", "addressBookIds": {"book": true},
                  "@type": "Card", "name": {"full": "Ada"}}]
    }, "c0"]], "sessionState": "s"}"#;

    /// A `ContactCard/set` answering every write accepted.
    const SET: &str = r#"{"methodResponses": [["ContactCard/set", {
        "accountId": "a1", "oldState": "s1", "newState": "s2",
        "updated": {"c1": null}, "destroyed": ["c1"]
    }, "c0"]], "sessionState": "s"}"#;

    const CARD: &[u8] = b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada Byron\r\nEND:VCARD\r\n";

    /// A write `--if-match` guards is compared with the hash of the card
    /// read before it, which an update reads as its patch base anyway: a
    /// stale one is refused `precondition-failed` before the set, the one
    /// a read reports lets it through.
    #[test]
    fn a_write_is_guarded_by_the_hash_of_the_card_it_reads() {
        use pimalaya_cli::error::code_of;

        use crate::error::ErrorCode;

        let (mut backend, _) = scripted(&[GET]);
        let etag = backend.get_card("book", "c1").unwrap().etag.unwrap();

        let stale = Some(ErrorCode::PreconditionFailed);
        let (mut backend, requests) = scripted(&[GET]);
        let err = backend
            .update_card("book", "c1", CARD.to_vec(), Some("0"))
            .unwrap_err();
        assert_eq!(code_of(&err), stale, "{err:#}");
        assert_eq!(requests.lock().unwrap().len(), 1);

        let (mut backend, requests) = scripted(&[GET, SET]);
        backend
            .update_card("book", "c1", CARD.to_vec(), Some(&etag))
            .unwrap();
        assert_eq!(requests.lock().unwrap().len(), 2);

        let (mut backend, requests) = scripted(&[GET]);
        let err = backend.delete_card("book", "c1", Some("0")).unwrap_err();
        assert_eq!(code_of(&err), stale, "{err:#}");
        assert_eq!(requests.lock().unwrap().len(), 1);

        let (mut backend, requests) = scripted(&[GET, SET]);
        backend.delete_card("book", "c1", Some(&etag)).unwrap();
        assert_eq!(requests.lock().unwrap().len(), 2);
    }

    /// A card the server does not hold reads as cardamum words a card not
    /// found, the server's answer in `sources`.
    #[test]
    fn a_missing_card_reads_as_a_card_not_found() {
        use pimalaya_cli::error::code_of;

        use crate::error::ErrorCode;

        let missing = r#"{"methodResponses": [["ContactCard/get", {
            "accountId": "a1", "state": "s1", "list": [], "notFound": ["c9"]
        }, "c0"]], "sessionState": "s"}"#;
        let (mut backend, _) = scripted(&[missing]);
        let err = backend.get_card("book", "c9").unwrap_err();

        assert_eq!(code_of(&err), Some(ErrorCode::CardNotFound));
        assert_eq!(
            err.to_string(),
            "Card `c9` not found in address book `book`"
        );
        assert!(err.chain().nth(1).is_some(), "{err:#}");
    }

    /// A card the server holds outside the named address book is a card
    /// not found there: a read does not print it under that book, an
    /// update and a delete send no `ContactCard/set`.
    #[test]
    fn a_card_of_another_address_book_is_not_found() {
        use pimalaya_cli::error::code_of;

        use crate::error::ErrorCode;

        let missing = Some(ErrorCode::CardNotFound);

        let (mut backend, _) = scripted(&[GET]);
        let err = backend.get_card("other", "c1").unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
        assert_eq!(
            err.to_string(),
            "Card `c1` not found in address book `other`"
        );

        let (mut backend, requests) = scripted(&[GET, SET]);
        let err = backend
            .update_card("other", "c1", CARD.to_vec(), None)
            .unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
        assert_eq!(requests.lock().unwrap().len(), 1);

        let (mut backend, requests) = scripted(&[GET, SET]);
        let err = backend.delete_card("other", "c1", None).unwrap_err();
        assert_eq!(code_of(&err), missing, "{err:#}");
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    /// An `AddressBook/get` answering `ids` as found, the others not.
    fn books(found: &[&str], not_found: &[&str]) -> String {
        let list: Vec<_> = found
            .iter()
            .map(|id| serde_json::json!({"id": id, "name": id}))
            .collect();
        let args = serde_json::json!({
            "accountId": "a1", "state": "s1", "list": list, "notFound": not_found,
        });
        serde_json::json!({"methodResponses": [["AddressBook/get", args, "c0"]], "sessionState": "s"})
            .to_string()
    }

    /// A listing of an address book the account does not hold fails
    /// `addressbook-not-found` naming it, whether the server refuses the
    /// filter or answers nothing; an empty one it holds lists empty.
    #[test]
    fn an_unknown_address_book_is_not_found() {
        use pimalaya_cli::error::code_of;

        use crate::error::ErrorCode;

        let refused = r#"{"methodResponses": [["error", {
            "type": "invalidArguments", "arguments": ["filter/inAddressBook"]
        }, "c0"]], "sessionState": "s"}"#;
        let empty = r#"{"methodResponses": [
            ["ContactCard/query", {"accountId": "a1", "queryState": "q", "ids": [],
                "position": 0, "canCalculateChanges": false}, "c0"],
            ["ContactCard/get", {"accountId": "a1", "state": "s1", "list": [],
                "notFound": []}, "c1"]
        ], "sessionState": "s"}"#;
        let unknown = books(&[], &["nope"]);
        let missing = Some(ErrorCode::AddressbookNotFound);

        for query in [refused, empty] {
            let (mut backend, requests) = scripted(&[query, &unknown]);
            let err = backend.list_cards("nope", 0, None).unwrap_err();

            assert_eq!(code_of(&err), missing, "{err:#}");
            assert_eq!(err.to_string(), "Addressbook `nope` not found");
            assert!(err.chain().nth(1).is_some(), "{err:#}");
            assert_eq!(requests.lock().unwrap().len(), 2);
        }

        let (mut backend, _) = scripted(&[empty, &books(&["RBk"], &[])]);
        let listed = backend.list_cards("RBk", 0, None).unwrap();
        assert!(listed.cards.is_empty() && listed.errors.is_empty());

        let (mut backend, _) = scripted(&[refused, &books(&["RBk"], &[])]);
        let err = backend.list_cards("RBk", 0, None).unwrap_err();
        assert_eq!(code_of::<ErrorCode>(&err), None, "{err:#}");
    }

    /// The address book the server states is the default says so.
    #[test]
    fn the_default_address_book_is_the_one_the_server_states() {
        let get = r#"{"methodResponses": [["AddressBook/get", {
            "accountId": "a1", "state": "s1", "notFound": [], "list": [
                {"id": "RBk", "name": "Personal", "isDefault": true},
                {"id": "B2", "name": "Work", "isDefault": false}
            ]
        }, "c0"]], "sessionState": "s"}"#;
        let (mut backend, _) = scripted(&[get]);
        let books = backend.list_addressbooks().unwrap();

        let defaults: Vec<_> = books.iter().map(|book| (&*book.id, book.default)).collect();
        assert_eq!(defaults, [("RBk", true), ("B2", false)]);
    }

    #[test]
    fn basic_auth_is_standard_padded_base64() {
        let auth = JmapAuthConfig::Basic {
            username: String::from("user"),
            password: Secret::Raw(SecretString::from("pa\u{e9}ss")),
        };
        let header = jmap_http_auth(auth, &mut SecretResolver::new()).unwrap();
        assert_eq!(header.expose_secret(), "Basic dXNlcjpwYcOpc3M=");
    }

    /// A card that does not convert is an entry, the other cards listed;
    /// a later page repeats no entry, the query reading every card.
    #[test]
    fn a_card_that_does_not_project_is_an_entry_the_others_listed() {
        use io_jmap::rfc9610::contact_card::JmapContactCard;
        use serde_json::json;

        use super::project_cards;
        use crate::error::ErrorCode;

        let card = |id: Option<&str>, name: &str| JmapContactCard {
            id: id.map(String::from),
            address_book_ids: Default::default(),
            card: json!({"@type": "Card", "name": {"full": name}})
                .as_object()
                .unwrap()
                .clone(),
        };
        let cards = || {
            vec![
                card(Some("c1"), "Ada"),
                card(None, "Nobody"),
                card(Some("c3"), "Bob"),
            ]
        };

        let first = project_cards("book", cards(), 0, Some(2));
        let ids: Vec<&str> = first.cards.iter().map(|card| card.id.as_str()).collect();
        assert_eq!(ids, ["c1", "c3"]);
        assert_eq!(first.errors.len(), 1);
        assert_eq!(first.errors[0].id, None);
        assert_eq!(first.errors[0].code, Some(ErrorCode::CardUnreadable));
        assert_eq!(
            first.errors[0].sources,
            ["JMAP ContactCard is missing its id"]
        );

        let second = project_cards("book", cards(), 1, Some(2));
        assert_eq!(second.cards.len(), 1);
        assert!(second.errors.is_empty());
    }
}
