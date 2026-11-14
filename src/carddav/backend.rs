//! # CardDAV backend
//!
//! Maps the unified addressbook and card operations onto
//! [`io_webdav::client::WebdavClientStd`] calls (RFC 6352).

use std::collections::BTreeSet;

use anyhow::{Context, Error, Result, bail};
use io_webdav::{
    client::{WebdavClientStd, WebdavClientStdError},
    rfc4918::{send::WebdavSendError, typed::WebdavUnreadableEntry},
    rfc6352::{
        addressbook::{CarddavAddressbook, CarddavAddressbookPatch},
        card::{CarddavCardEntry, list::CarddavCardListOptions},
    },
};
use pimalaya_config::secret::SecretResolver;

use crate::{
    carddav::client::{is_missing, row_code},
    config::CarddavConfig,
    error::{self, ErrorCode, ItemError},
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
        client::{CardWindow, first_page, window},
        uuid::uuid_v4,
    },
};

/// CardDAV arm of the unified-API client.
///
/// Wraps a connected io-webdav client whose addressbook home-set is
/// already resolved.
pub struct CarddavBackend {
    pub inner: WebdavClientStd,
}

impl CarddavBackend {
    /// Opens the client from the account's `[carddav]` block.
    ///
    /// Discovery runs when the block needs it, see
    /// [`open_carddav_client`](crate::carddav::client::open_carddav_client).
    pub fn new(config: CarddavConfig, resolver: &mut SecretResolver) -> Result<Self> {
        let inner = crate::carddav::client::open_carddav_client(config, resolver)?;
        Ok(Self { inner })
    }

    /// Lists every addressbook under the discovered home-set.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let addressbooks = self.inner.list_addressbooks()?;
        Ok(addressbooks.into_iter().map(into_addressbook).collect())
    }

    /// Creates an addressbook collection under the home-set.
    ///
    /// The given name doubles as the URL segment, and comes back as the
    /// new addressbook id.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if name.is_empty() {
            bail!("Addressbook name cannot be empty");
        }

        let wire = CarddavAddressbook {
            id: name.to_string(),
            display_name: Some(name.to_string()),
            description: description.map(str::to_string),
            color: color.map(str::to_string),
            ctag: None,
            sync_token: None,
            supported_reports: BTreeSet::new(),
        };
        self.inner.create_addressbook(&wire)?;

        Ok(name.to_string())
    }

    /// Applies `patch` to the addressbook identified by `id`.
    ///
    /// The diff maps one-to-one onto a `PROPPATCH`: a field left unset
    /// is absent from the request and keeps its server value, while a
    /// cleared one leaves as a `DAV:remove` instruction.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        let patch = CarddavAddressbookPatch {
            id: id.to_string(),
            display_name: patch.name.map(Some),
            description: patch.description,
            color: patch.color,
        };

        self.inner.update_addressbook(&patch)?;
        Ok(())
    }

    /// Deletes the addressbook collection identified by `id`.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        self.inner.delete_addressbook(id)?;
        Ok(())
    }

    /// Lists the `take` cards inside `addressbook_id` after the first `skip`.
    ///
    /// Each row the server answered without the card's data is an entry, as
    /// is a listing the server truncated, on the first page only since the
    /// whole address book is read before the page is cut.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<CardWindow> {
        let opts = CarddavCardListOptions::default();
        let listed = self.inner.list_cards(addressbook_id, &opts)?;

        let cards = listed
            .cards
            .into_iter()
            .map(|entry| into_card(addressbook_id, entry))
            .collect();

        let mut errors: Vec<ItemError> =
            listed.unreadable.into_iter().map(unreadable_row).collect();

        if listed.truncated {
            errors.push(truncated());
        }

        Ok(CardWindow {
            cards: window(cards, skip, take),
            errors: first_page(errors, skip),
        })
    }

    /// Fetches `card_id` from `addressbook_id`.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let body = self
            .inner
            .read_card(addressbook_id, card_id)
            .map_err(|err| card_error(card_id, err))?;

        Ok(Card {
            id: card_id.to_string(),
            addressbook_id: addressbook_id.to_string(),
            etag: body.etag,
            contents: body.data,
        })
    }

    /// Creates a card inside `addressbook_id` under a fresh UUID name.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let id = fresh_card_id()?;
        let created = self
            .inner
            .create_card(addressbook_id, &id, contents)
            .map_err(card_write_error)?;
        Ok(created.id)
    }

    /// Overwrites `card_id` inside `addressbook_id`, gated on `if_match`.
    ///
    /// A `PUT` is create-or-replace, so without a precondition an
    /// unknown id quietly creates a card. A missing ETag is filled from
    /// a read, since iCloud 412s the `If-Match: *` that would say the
    /// same in one request.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        let if_match = match if_match {
            Some(etag) => Some(etag.to_string()),
            None => {
                self.inner
                    .read_card(addressbook_id, card_id)
                    .map_err(|err| card_error(card_id, err))
                    .with_context(|| {
                        format!("Cannot read the current version of card `{card_id}`")
                    })?
                    .etag
            }
        };

        self.inner
            .update_card(addressbook_id, card_id, contents, if_match.as_deref())
            .map_err(card_write_error)?;
        Ok(CardUpdateOutcome::default())
    }

    /// Permanently deletes `card_id` from `addressbook_id`.
    pub fn delete_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.inner
            .delete_card(addressbook_id, card_id, if_match)
            .map_err(|err| card_error(card_id, err))?;
        Ok(())
    }
}

/// Maps a wire addressbook to the shared shape, id as fallback name.
fn into_addressbook(wire: CarddavAddressbook) -> Addressbook {
    let name = wire.display_name.unwrap_or_else(|| wire.id.clone());

    Addressbook {
        id: wire.id,
        name,
        description: wire.description,
        color: wire.color,
        default: false,
    }
}

/// Maps a WebDAV [`CarddavCardEntry`] to a shared [`Card`].
fn into_card(addressbook_id: &str, entry: CarddavCardEntry) -> Card {
    Card {
        id: entry.id,
        addressbook_id: addressbook_id.to_string(),
        etag: entry.etag,
        contents: entry.data,
    }
}

/// The entry of a listed card the server answered without its data: coded
/// from the row's status or the status refusing its properties, a 403
/// `permission-denied`, else `card-unreadable`; each refused property in its
/// sources.
fn unreadable_row(row: WebdavUnreadableEntry) -> ItemError {
    let code = row
        .status
        .into_iter()
        .chain(row.failures.iter().map(|failure| failure.status))
        .find_map(row_code)
        .unwrap_or(ErrorCode::CardUnreadable);

    let error = match row.status {
        Some(status) => format!("The server refused the card with status {status}"),
        None if !row.failures.is_empty() => String::from("The server refused the card data"),
        None => String::from("The server sent the card without its data"),
    };

    ItemError {
        id: Some(row.id),
        code: Some(code),
        error,
        sources: row.failures.iter().map(ToString::to_string).collect(),
    }
}

/// The entry of a listing the server truncated (a 507 row, RFC 6352
/// §8.6.1): no card to name and no code, the listing incomplete.
fn truncated() -> ItemError {
    ItemError {
        id: None,
        code: None,
        error: String::from("The server truncated the listing: more cards exist than it returned"),
        sources: Vec::new(),
    }
}

/// Adds a hint when the server rejects a card as an invalid vCard.
///
/// The CardDAV `valid-address-data` precondition (RFC 6352 §6.3.2.1).
/// cardamum forwards the vCard unchanged, and providers disagree on
/// what they accept: most require a `UID`, some (iCloud) also an `N`.
///
/// The server's answer stays the cause, its typed error kept for the
/// readers of [`crate::error::code`].
fn card_write_error(err: WebdavClientStdError) -> Error {
    let WebdavClientStdError::Send(WebdavSendError::HttpStatus { status: 403, body }) = &err else {
        return err.into();
    };

    let lower = body.to_ascii_lowercase();
    if !(lower.contains("valid-address-data") || lower.contains("vcard")) {
        return err.into();
    }

    Error::new(err).context(
        "The server rejected the vCard as invalid. cardamum sends the vCard \
         you provide unchanged, and providers disagree on what they accept: \
         most require a UID property, and some (e.g. iCloud) also require an N \
         property",
    )
}

/// The failure of a request on the card `card_id`, `card-not-found` when
/// the server holds no such card.
fn card_error(card_id: &str, err: WebdavClientStdError) -> Error {
    match is_missing(&err) {
        true => error::card_not_found(card_id, err.into()),
        false => err.into(),
    }
}

/// Generates a fresh card resource name, a random UUIDv4 plus `.vcf`.
///
/// CardDAV makes the caller name the resource, and io-webdav uses that
/// name verbatim (it never adds nor strips an extension), so the caller
/// owns the whole of it.
fn fresh_card_id() -> Result<String> {
    Ok(format!("{}.vcf", uuid_v4()?))
}

#[cfg(test)]
mod tests {
    use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
    use pimalaya_cli::error::code_of;
    use url::Url;

    use super::CarddavBackend;
    use crate::{
        error::{self, ErrorCode},
        script::{self, Script},
    };

    const CARD: &[u8] = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:c1\r\nFN:Jane\r\nEND:VCARD\r\n";

    /// The backend over a CardDAV server answering `status` with `body`,
    /// its home set already discovered.
    fn answering(status: u16, body: &str) -> CarddavBackend {
        let (script, _) = Script::new([script::response(status, "application/xml", body)]);
        let base = Url::parse("https://dav.example.org/").unwrap();
        let home = base.join("/dav/").unwrap();
        let inner =
            WebdavClientStd::from_parts(script, WebdavAuth::None, base, None, None, Some(home));
        CarddavBackend { inner }
    }

    /// The code a write refused with `status` and `body` prints in its
    /// failure object.
    fn create_code(status: u16, body: &str) -> Option<ErrorCode> {
        let err = answering(status, body)
            .create_card("book", CARD.to_vec())
            .unwrap_err();
        let code = error::code(&err);
        assert_eq!(code_of(&error::with_code(err)), code);
        code
    }

    /// A refusal is coded from its status and the precondition its body
    /// names: a quota, a rate, or the account's rights, never a card the
    /// server would not take.
    #[test]
    fn a_carddav_refusal_carries_its_code() {
        let privileges =
            r#"<?xml version="1.0"?><D:error xmlns:D="DAV:"><D:need-privileges/></D:error>"#;
        let quota =
            r#"<?xml version="1.0"?><D:error xmlns:D="DAV:"><D:quota-not-exceeded/></D:error>"#;
        let uid = r#"<?xml version="1.0"?><D:error xmlns:D="DAV:"><C:no-uid-conflict xmlns:C="urn:ietf:params:xml:ns:carddav"/></D:error>"#;

        assert_eq!(create_code(403, ""), Some(ErrorCode::PermissionDenied));
        assert_eq!(
            create_code(403, privileges),
            Some(ErrorCode::PermissionDenied)
        );
        assert_eq!(create_code(403, quota), Some(ErrorCode::QuotaExceeded));
        assert_eq!(create_code(507, ""), Some(ErrorCode::QuotaExceeded));
        assert_eq!(create_code(429, ""), Some(ErrorCode::RateLimited));
        assert_eq!(create_code(403, uid), None);
        assert_eq!(create_code(500, ""), None);
    }

    /// A vCard the server refuses keeps the server's answer as the cause
    /// of the hint, a matter of the card and not of rights.
    #[test]
    fn a_refused_vcard_keeps_the_answer_as_its_cause() {
        let body = r#"<?xml version="1.0"?><D:error xmlns:D="DAV:"><C:valid-address-data xmlns:C="urn:ietf:params:xml:ns:carddav"/></D:error>"#;
        let err = answering(403, body)
            .create_card("book", CARD.to_vec())
            .unwrap_err();

        assert!(err.to_string().starts_with("The server rejected the vCard"));
        let cause = err.chain().nth(1).unwrap().to_string();
        assert!(cause.contains("403"), "{cause}");
        assert_eq!(error::code(&err), None, "{err:#}");
    }

    /// A refused write fails with an object coded by the refusal, its
    /// message and causes printing as the backend wrote them.
    #[test]
    fn a_refused_write_prints_its_code_in_its_failure_object() {
        use pimalaya_cli::error::ErrorReport;

        let err = answering(403, "")
            .delete_card("book", "c1.vcf", None)
            .unwrap_err()
            .context("Delete card `c1.vcf`");
        let text = format!("{err:#}");

        let report = ErrorReport::<ErrorCode>::from(error::with_code(err));
        let object = serde_json::to_value(&report).unwrap();
        assert_eq!(object["code"], "permission-denied", "{object}");

        let mut chain = vec![object["error"].as_str().unwrap().to_owned()];
        for source in object["sources"].as_array().unwrap() {
            chain.push(source.as_str().unwrap().to_owned());
        }
        assert_eq!(chain.join(": "), text);
    }

    /// A card the address book does not hold is a card not found, on a
    /// read as on a delete.
    #[test]
    fn a_missing_card_is_a_card_not_found() {
        let err = answering(404, "").get_card("book", "c9.vcf").unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::CardNotFound), "{err:#}");

        let err = answering(410, "")
            .delete_card("book", "c9.vcf", None)
            .unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::CardNotFound), "{err:#}");
    }

    /// A listing answering a card, a row refused whole, a row refusing its
    /// data, a row missing it and a truncation row.
    const LISTING: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:carddav">
  <d:response>
    <d:href>/dav/book/c1.vcf</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"e1"</d:getetag>
        <c:address-data>BEGIN:VCARD&#13;
VERSION:4.0&#13;
UID:c1&#13;
FN:Jane&#13;
END:VCARD&#13;
</c:address-data>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/dav/book/c2.vcf</d:href>
    <d:status>HTTP/1.1 403 Forbidden</d:status>
  </d:response>
  <d:response>
    <d:href>/dav/book/c3.vcf</d:href>
    <d:propstat>
      <d:prop><d:getetag>"e3"</d:getetag></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
    <d:propstat>
      <d:prop><c:address-data/></d:prop>
      <d:status>HTTP/1.1 403 Forbidden</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/dav/book/c4.vcf</d:href>
    <d:propstat>
      <d:prop><d:getetag>"e4"</d:getetag></d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/dav/book/</d:href>
    <d:status>HTTP/1.1 507 Insufficient Storage</d:status>
  </d:response>
</d:multistatus>"#;

    /// Each row the server refused is an entry naming its card, a 403
    /// `permission-denied`, its refused properties in its sources; the
    /// truncation an entry naming no card and carrying no code.
    #[test]
    fn a_carddav_listing_names_the_rows_it_could_not_read() {
        let listed = answering(207, LISTING).list_cards("book", 0, None).unwrap();

        let ids: Vec<&str> = listed.cards.iter().map(|card| card.id.as_str()).collect();
        assert_eq!(ids, ["c1.vcf"]);

        let entries: Vec<_> = listed
            .errors
            .iter()
            .map(|entry| {
                let id = entry.id.as_deref();
                let sources: Vec<&str> = entry.sources.iter().map(String::as_str).collect();
                (id, entry.code, entry.error.as_str(), sources)
            })
            .collect();

        assert_eq!(
            entries,
            [
                (
                    Some("c2.vcf"),
                    Some(ErrorCode::PermissionDenied),
                    "The server refused the card with status 403",
                    vec![],
                ),
                (
                    Some("c3.vcf"),
                    Some(ErrorCode::PermissionDenied),
                    "The server refused the card data",
                    vec!["address-data (403)"],
                ),
                (
                    Some("c4.vcf"),
                    Some(ErrorCode::CardUnreadable),
                    "The server sent the card without its data",
                    vec![],
                ),
                (
                    None,
                    None,
                    "The server truncated the listing: more cards exist than it returned",
                    vec![],
                ),
            ]
        );
    }

    /// The whole address book is read before the page is cut, so the
    /// entries print on the first page only.
    #[test]
    fn a_carddav_listing_prints_its_entries_on_the_first_page_only() {
        let listed = answering(207, LISTING)
            .list_cards("book", 1, Some(2))
            .unwrap();

        assert!(listed.cards.is_empty());
        assert!(listed.errors.is_empty());
    }
}
