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
            query::{
                JmapContactCardFilter, JmapContactCardQueryOptions, JmapContactCardQueryOutput,
            },
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
        client::CardWindow,
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
    /// The query asks for that window alone (`position`, `limit`), in
    /// chunks no wider than the session's `maxObjectsInGet`, the chained
    /// `ContactCard/get` fetching what the query found. A server answering
    /// fewer cards than asked is asked again from where it stopped while
    /// its `total` says more follow, or, giving none, while it answered
    /// every card asked. A card that does not convert to a vCard is left
    /// out, an entry naming it instead, on the page holding it.
    ///
    /// A first query that fails or finds nothing is checked against the
    /// address books, one `AddressBook/get`, so an unknown one fails
    /// `addressbook-not-found` naming it rather than reading as empty or as
    /// the server's refusal of the filter.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<CardWindow> {
        let max_get = self
            .inner
            .session()
            .and_then(|session| session.core_capability().max_objects_in_get)
            .filter(|max| *max > 0);
        let mut places = Vec::new();
        let mut position = skip as u64;

        loop {
            let left = take.map(|take| take.saturating_sub(places.len()) as u64);
            let limit = match (left, max_get) {
                (Some(left), Some(max)) => Some(left.min(max)),
                (left, max) => left.or(max),
            };
            let first = places.is_empty() && position == skip as u64;
            let out = self.query_cards(addressbook_id, position, limit, first)?;

            let listed = (out.cards.len() + out.not_found.len()) as u64;
            position += listed;
            places.extend(out.cards.into_iter().map(Some));
            places.extend(out.not_found.iter().map(|_| None));

            let done = listed == 0
                || left.is_some_and(|left| listed >= left)
                || match out.total {
                    Some(total) => position >= total,
                    None => limit.is_none_or(|limit| listed < limit),
                };

            if done {
                break;
            }
        }

        Ok(project_cards(addressbook_id, places, take))
    }

    /// Queries the AddressBook's cards from `position`, `limit` at most,
    /// the `first` query of a listing checked against the address books
    /// when it fails or finds nothing.
    fn query_cards(
        &mut self,
        addressbook_id: &str,
        position: u64,
        limit: Option<u64>,
        first: bool,
    ) -> Result<JmapContactCardQueryOutput> {
        let opts = JmapContactCardQueryOptions {
            filter: Some(JmapContactCardFilter {
                in_address_book: Some(addressbook_id.to_string()),
                ..Default::default()
            }),
            position: Some(position),
            limit,
            ..Default::default()
        };

        match self.inner.contact_card_query(opts) {
            Ok(out) if !first || !out.cards.is_empty() => Ok(out),
            Err(err) if !first => Err(err.into()),
            queried => match (self.holds_addressbook(addressbook_id)?, queried) {
                (true, queried) => Ok(queried?),
                (false, Ok(_)) => {
                    let cause = anyhow!("JMAP AddressBook/get answered the id as `notFound`");
                    Err(error::addressbook_not_found(addressbook_id, cause))
                }
                (false, Err(err)) => Err(error::addressbook_not_found(addressbook_id, err.into())),
            },
        }
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
    ///
    /// A create refused with no code of its own, or `notFound`, is checked
    /// against the address books, one `AddressBook/get`, so a card added
    /// to an unknown one fails `addressbook-not-found` naming it, the
    /// server's refusal in `sources`.
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
            let refusal = card_write_error("create", &err, &unmapped);

            if matches!(err.code(), None | Some(ErrorCode::CardNotFound))
                && !self.holds_addressbook(addressbook_id)?
            {
                return Err(error::addressbook_not_found(addressbook_id, refusal));
            }

            return Err(refusal);
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

/// The queried `places` as vCard documents of `addressbook_id`, beside an
/// entry for each card that does not convert, `None` standing for a card
/// the query found and the get did not.
///
/// Every place holds one card of the window, so the page counts it: a
/// card that does not convert, or that the get did not find, holds it
/// with a [`placeholder`], as does the card past a page of `take` minus
/// one, which only tells that another follows and is never converted.
fn project_cards(
    addressbook_id: &str,
    places: Vec<Option<JmapContactCard>>,
    take: Option<usize>,
) -> CardWindow {
    let mut cards = Vec::new();
    let mut errors = Vec::new();

    for (index, place) in places.into_iter().enumerate() {
        let Some(card) = place else {
            cards.push(placeholder(addressbook_id, None));
            continue;
        };
        let id = card.id.clone();

        if take.is_some_and(|take| index + 1 >= take) {
            cards.push(placeholder(addressbook_id, id));
            continue;
        }

        match project::to_card(addressbook_id, card) {
            Ok(card) => cards.push(card),
            Err(err) => {
                let err =
                    Error::msg(err).context("Cannot convert the JMAP contact card to a vCard");
                errors.push(error::unreadable_card(id.as_deref(), err));
                cards.push(placeholder(addressbook_id, id));
            }
        }
    }

    CardWindow { cards, errors }
}

/// The card holding the place of one the page leaves out: its `id`,
/// empty when it has none, and no contents.
fn placeholder(addressbook_id: &str, id: Option<String>) -> Card {
    Card {
        id: id.unwrap_or_default(),
        addressbook_id: addressbook_id.to_string(),
        etag: None,
        contents: Vec::new(),
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
    use serde_json::Value;

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

    /// A card that does not convert is an entry, the other cards listed,
    /// and keeps its place in the window, as does the card past the page,
    /// never converted, and a card the get did not find.
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
        let places = vec![
            Some(card(Some("c1"), "Ada")),
            Some(card(None, "Nobody")),
            None,
            Some(card(Some("c4"), "Bob")),
            Some(card(None, "Next")),
        ];

        let window = project_cards("book", places, Some(5));
        let ids: Vec<&str> = window.cards.iter().map(|card| card.id.as_str()).collect();
        assert_eq!(ids, ["c1", "", "", "c4", ""]);
        let held: Vec<bool> = window.cards.iter().map(|c| c.contents.is_empty()).collect();
        assert_eq!(held, [false, true, true, false, true]);
        assert_eq!(window.errors.len(), 1);
        assert_eq!(window.errors[0].id, None);
        assert_eq!(window.errors[0].code, Some(ErrorCode::CardUnreadable));
        assert_eq!(
            window.errors[0].sources,
            ["JMAP ContactCard is missing its id"]
        );
    }

    /// The backend over a JMAP server whose session states `max_get` as
    /// its `maxObjectsInGet`, answering each request with the next of
    /// `bodies`.
    fn scripted_limited(
        max_get: u64,
        bodies: &[String],
    ) -> (super::JmapBackend, crate::script::Requests) {
        let bodies: Vec<&str> = bodies.iter().map(String::as_str).collect();
        let (mut backend, requests) = scripted(&bodies);
        let core = serde_json::json!({"maxObjectsInGet": max_get});
        let session = backend.inner.session.as_mut().unwrap();
        session
            .capabilities
            .insert("urn:ietf:params:jmap:core".into(), core);
        (backend, requests)
    }

    /// A `ContactCard/query` and its `ContactCard/get` answering the cards
    /// `ids` from `position`, out of `total` when given.
    fn query_page(ids: &[&str], position: usize, total: Option<usize>) -> String {
        use serde_json::json;

        let list: Vec<_> = ids
            .iter()
            .map(|id| {
                json!({"id": id, "addressBookIds": {"book": true},
                       "@type": "Card", "name": {"full": id}})
            })
            .collect();
        let mut query = json!({
            "accountId": "a1", "queryState": "q", "ids": ids,
            "position": position, "canCalculateChanges": false,
        });
        if let Some(total) = total {
            query["total"] = json!(total);
        }
        let get = json!({"accountId": "a1", "state": "s1", "list": list, "notFound": []});
        json!({"methodResponses": [
            ["ContactCard/query", query, "c0"],
            ["ContactCard/get", get, "c1"],
        ], "sessionState": "s"})
        .to_string()
    }

    /// The `position` and `limit` each `ContactCard/query` a server
    /// received asked for.
    fn query_windows(requests: &crate::script::Requests) -> Vec<(Value, Value)> {
        let requests = requests.lock().unwrap();
        requests
            .iter()
            .map(|request| {
                let request = String::from_utf8_lossy(request);
                let (_, body) = request.split_once("\r\n\r\n").unwrap();
                let body: Value = serde_json::from_str(body).unwrap();
                let args = &body["methodCalls"][0][1];
                (args["position"].clone(), args["limit"].clone())
            })
            .collect()
    }

    /// A page asks the query for its own window alone, the page and the
    /// card telling whether another follows, from the cards before it.
    #[test]
    fn a_page_queries_its_window_alone() {
        use serde_json::json;

        let (mut backend, requests) = scripted(&[&query_page(&["c3", "c4", "c5"], 2, Some(9))]);
        let window = backend.list_cards("book", 2, Some(3)).unwrap();

        let ids: Vec<&str> = window.cards.iter().map(|card| card.id.as_str()).collect();
        assert_eq!(ids, ["c3", "c4", "c5"]);
        assert_eq!(query_windows(&requests), [(json!(2), json!(3))]);
    }

    /// A page wider than the session's `maxObjectsInGet` is read in
    /// chunks no wider, and a server answering fewer cards than asked is
    /// asked again from where it stopped while its `total` says more
    /// follow.
    #[test]
    fn a_page_is_read_in_chunks_the_server_takes() {
        use serde_json::json;

        let bodies = [
            query_page(&["c1", "c2"], 0, Some(9)),
            query_page(&["c3"], 2, Some(9)),
        ];
        let (mut backend, requests) = scripted_limited(2, &bodies);
        let window = backend.list_cards("book", 0, Some(3)).unwrap();
        assert_eq!(window.cards.len(), 3);
        assert_eq!(
            query_windows(&requests),
            [(json!(0), json!(2)), (json!(2), json!(1))]
        );

        let bodies = [
            query_page(&["c1"], 0, Some(9)),
            query_page(&["c2", "c3"], 1, Some(9)),
        ];
        let (mut backend, requests) = scripted(&bodies.each_ref().map(String::as_str));
        let window = backend.list_cards("book", 0, Some(3)).unwrap();
        assert_eq!(window.cards.len(), 3);
        assert_eq!(
            query_windows(&requests),
            [(json!(0), json!(3)), (json!(1), json!(2))]
        );

        let bodies = [
            query_page(&["c1", "c2"], 0, Some(3)),
            query_page(&["c3"], 2, Some(3)),
        ];
        let (mut backend, requests) = scripted_limited(2, &bodies);
        let window = backend.list_cards("book", 0, None).unwrap();
        assert_eq!(window.cards.len(), 3);
        assert_eq!(
            query_windows(&requests),
            [(json!(0), json!(2)), (json!(2), json!(2))]
        );
    }

    /// A card added to an address book the account does not hold is
    /// `addressbook-not-found` naming it, whatever SetError the server
    /// refuses the create with, as the other verbs answer an unknown one;
    /// one it holds keeps the server's refusal as it is.
    #[test]
    fn a_card_added_to_an_unknown_address_book_is_not_found() {
        use pimalaya_cli::error::code_of;

        use crate::error::ErrorCode;

        let refused = |set_error: &str| {
            format!(
                r#"{{"methodResponses": [["ContactCard/set", {{
                    "accountId": "a1", "oldState": "s1", "newState": "s1",
                    "notCreated": {{"c0": {set_error}}}
                }}, "c0"]], "sessionState": "s"}}"#
            )
        };
        let invalid = refused(r#"{"type": "invalidProperties", "properties": ["addressBookIds"]}"#);
        let not_found = refused(r#"{"type": "notFound"}"#);

        for set in [&invalid, &not_found] {
            let (mut backend, requests) = scripted(&[set, &books(&[], &["nope"])]);
            let err = backend.create_card("nope", CARD.to_vec()).unwrap_err();

            assert_eq!(
                code_of(&err),
                Some(ErrorCode::AddressbookNotFound),
                "{err:#}"
            );
            assert_eq!(err.to_string(), "Addressbook `nope` not found");
            assert!(format!("{err:#}").contains("rejected"), "{err:#}");
            assert_eq!(requests.lock().unwrap().len(), 2);
        }

        let (mut backend, _) = scripted(&[&invalid, &books(&["RBk"], &[])]);
        let err = backend.create_card("RBk", CARD.to_vec()).unwrap_err();
        assert_eq!(crate::error::code(&err), None, "{err:#}");
    }

    /// The backend over a JMAP server answering each request with the
    /// next of `responses`, whole HTTP answers.
    fn answering(responses: Vec<Vec<u8>>) -> super::JmapBackend {
        let (mut backend, _) = scripted(&[]);
        let session = backend.inner.session.take().unwrap();
        let (script, _) = crate::script::Script::new(responses);
        let auth = SecretString::from("Bearer token");
        let inner = io_jmap::client::JmapClientStd::from_parts(script, auth, session);
        super::JmapBackend { inner }
    }

    /// A listing the server refuses carries the code of its refusal: an
    /// HTTP 403 `permission-denied`, a 429 `rate-limited`, as does a
    /// `forbidden` method error answering the query of an address book
    /// the account holds.
    #[test]
    fn a_refused_listing_carries_its_code() {
        use crate::{error::ErrorCode, script::response};

        let refusing = |status| {
            let refusal = response(status, "application/json", "{}");
            answering(vec![refusal.clone(), refusal])
        };

        let err = refusing(403).list_cards("RBk", 0, Some(3)).unwrap_err();
        let denied = Some(ErrorCode::PermissionDenied);
        assert_eq!(crate::error::code(&err), denied, "{err:#}");

        let err = refusing(429).list_cards("RBk", 0, Some(3)).unwrap_err();
        let limited = Some(ErrorCode::RateLimited);
        assert_eq!(crate::error::code(&err), limited, "{err:#}");

        let forbidden = r#"{"methodResponses": [["error", {
            "type": "forbidden"
        }, "c0"]], "sessionState": "s"}"#;
        let (mut backend, _) = scripted(&[forbidden, &books(&["RBk"], &[])]);
        let err = backend.list_cards("RBk", 0, Some(3)).unwrap_err();
        assert_eq!(crate::error::code(&err), denied, "{err:#}");
    }
}
