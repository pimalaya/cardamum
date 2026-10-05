//! # Pimdir backend
//!
//! Pimdir arm of the shared-API client, mapping the shared addressbook
//! and card operations onto a local pimdir store.
//!
//! Reads build cards from the stored contact summary (pimdir STORAGE
//! Annex A) plus the blob store: a card whose body is not local still
//! lists, and reading it reports "body not fetched" rather than erroring.
//!
//! Writes append one action to the store's queue (pimdir STORAGE §15.1)
//! through a producer opened for that write, the body first, then the row
//! pinning it. The store's owner, a sync, applies and pushes it, and the
//! reader folds the pending queue so a staged change shows here at once.

use std::io::Write;

use anyhow::{Result, anyhow, bail};
use io_pimdir::{
    client::reader::{PimdirCollection, PimdirItem},
    codec::PimdirAction,
    object::{PimdirHash, PimdirObject},
    placement::PimdirFlags,
    summary::{PimdirSummary, contact},
};
use log::warn;

use crate::{
    config::PimdirConfig,
    error::{CodedError, ErrorCode},
    pimdir::client::PimdirClient,
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
        client::paginate,
    },
};

/// The media type a pimdir collection carries to be an addressbook.
const CARD_KIND: &str = "text/vcard";

/// How many items to pull per keyset page when scanning a whole collection.
const SCAN_BATCH: usize = 500;

/// Pimdir backend of the shared-API client, over an opened local store.
pub struct PimdirBackend {
    inner: PimdirClient,
}

impl PimdirBackend {
    /// Opens the store from the account's `[pimdir]` block.
    pub fn new(config: PimdirConfig) -> Result<Self> {
        Ok(Self {
            inner: PimdirClient::new(config)?,
        })
    }

    /// Lists the contact collections as addressbooks, sorted by name.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let mut addressbooks: Vec<Addressbook> = self
            .collections()?
            .into_iter()
            .map(|collection| Addressbook {
                name: if collection.name.is_empty() {
                    collection.id.clone()
                } else {
                    collection.name.clone()
                },
                id: collection.id,
                description: collection.description,
                color: collection.color,
                default: collection.role.as_deref() == Some("default"),
            })
            .collect();

        addressbooks.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(addressbooks)
    }

    /// Always fails, a collection being the store owner's to declare.
    ///
    /// This backend is a producer (pimdir STORAGE §8), which appends item
    /// actions and nothing else, and a collection declared here would be
    /// one no sync knows about, so no address book would come of it.
    pub fn create_addressbook(
        &mut self,
        _name: &str,
        _description: Option<&str>,
        _color: Option<&str>,
    ) -> Result<String> {
        bail!(
            "The pimdir backend cannot create an addressbook: the collection row is \
             the sync's to write. Create it on the server and sync"
        )
    }

    /// Always fails, a collection's row being what a sync writes.
    ///
    /// This backend stages item actions only, so name, description and
    /// colour are refused rather than changed into something no sync would
    /// carry. A rename is worse still: io-pimdir renames the identifier, so
    /// `--name` would move every card under an id nobody asked for.
    pub fn update_addressbook(&mut self, _id: &str, _patch: AddressbookDiff) -> Result<()> {
        bail!(
            "The pimdir backend cannot update an addressbook: its name, description \
             and color come from the server through a sync. Rename it there and sync"
        )
    }

    /// Always fails, io-pimdir exposing no collection removal.
    ///
    /// The queue carries item actions only, so a delete here would be a
    /// silent no-op.
    pub fn delete_addressbook(&mut self, _id: &str) -> Result<()> {
        bail!(
            "The pimdir backend cannot delete an addressbook; \
             delete it on the server and sync, or remove the store directory"
        )
    }

    /// Lists the cards inside `addressbook_id`, 1-indexed paging.
    ///
    /// The order is the store's own contacts one, display name ascending.
    /// An unhydrated card lists as a preview projected from its stored
    /// summary, [`get_card`](Self::get_card) reporting the absence.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        self.known_collection(addressbook_id)?;

        let cards = self
            .scan_items(addressbook_id)?
            .into_iter()
            .map(|item| self.card_from_item(addressbook_id, item))
            .collect::<Result<Vec<_>>>()?;

        Ok(paginate(cards, page, page_size))
    }

    /// Fetches `card_id` from `addressbook_id`, reading its blob.
    ///
    /// A card not hydrated to `Full` has no local body, and fails with a
    /// clear "body not fetched", the cue to sync rather than a data-loss
    /// error.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        self.known_collection(addressbook_id)?;

        let item = self.item(addressbook_id, card_id)?;
        let Some(hash) = item.object else {
            return Err(CodedError::new(
                ErrorCode::BodyPending,
                format!(
                    "Card `{card_id}` in `{addressbook_id}` is not downloaded yet \
                     (body not fetched); run a sync to hydrate it"
                ),
            )
            .into());
        };
        let contents =
            self.inner.blobs.get(&hash)?.ok_or_else(|| {
                anyhow!("Body blob missing for `{card_id}` in `{addressbook_id}`")
            })?;

        Ok(Card {
            id: card_id.to_string(),
            addressbook_id: addressbook_id.to_string(),
            etag: Some(hash.0),
            contents,
        })
    }

    /// Stages a locally-authored card as an `add` action the next sync
    /// applies and uploads.
    ///
    /// Returns the card's link id, its `UID`: a queued create carries no
    /// public `seq` until the store's owner applies it, so there is no
    /// store-assigned id to report yet. The owner derives the summary from
    /// the body, and the key it derives is the one reported here.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        self.known_collection(addressbook_id)?;

        let link_id = contact::derive(&contents).link_id;

        self.stage(addressbook_id, &contents, |hash| PimdirAction::Add {
            link_id: Some(link_id.clone()),
            flags: PimdirFlags::default(),
            object: Some(hash),
        })?;

        Ok(link_id.0)
    }

    /// Stages a body replacement for `card_id` as an `update` action.
    ///
    /// The next sync applies and pushes it, three-way merging against the
    /// stored base. `if_match` gates the staging on the card's version,
    /// the store's hash of its body with the pending queue folded in: the
    /// gate is the caller's, the merge the store's.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        self.known_collection(addressbook_id)?;

        let item = self.item(addressbook_id, card_id)?;
        check_version(&item, addressbook_id, card_id, if_match)?;
        let seq = item.seq;

        self.stage(addressbook_id, &contents, |hash| PimdirAction::Update {
            seq,
            object: hash,
        })?;

        Ok(CardUpdateOutcome::default())
    }

    /// Stages a `remove` action for `card_id`.
    ///
    /// The next sync applies it as a tombstone and pushes a server-side
    /// delete. `if_match` gates it as it gates
    /// [`update_card`](Self::update_card).
    pub fn delete_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.known_collection(addressbook_id)?;

        let item = self.item(addressbook_id, card_id)?;
        check_version(&item, addressbook_id, card_id, if_match)?;
        let seq = item.seq;
        let action = PimdirAction::Remove { seq };
        let mut producer = self.inner.producer()?;
        let partials = producer
            .check(addressbook_id, &action)
            .map_err(|err| anyhow!("Stage the pimdir action: {err}"))?;
        producer
            .enqueue(addressbook_id, &action, None)
            .map_err(|err| anyhow!("Stage the pimdir action: {err}"))?;

        for partial in &partials {
            warn!("{partial}");
        }
        Ok(())
    }

    /// The address book collections of the configured account.
    ///
    /// One store holds every kind a sync caches (pimdir STORAGE §9.2), so
    /// the kind separates an address book from a mailbox or a calendar. A
    /// kind-less one counts: a sync predating kinds left the column empty.
    fn collections(&self) -> Result<Vec<PimdirCollection>> {
        let collections = match self.inner.account.as_deref() {
            Some(account) => self
                .inner
                .reader
                .list_collections_by_account(Some(account))?,
            None => self.inner.reader.list_collections()?,
        };

        Ok(collections
            .into_iter()
            .filter(|collection| collection.kind.is_empty() || collection.kind == CARD_KIND)
            .collect())
    }

    /// Pulls every live item of a collection by keyset paging, each with
    /// its summary joined.
    ///
    /// The order is the contacts one the store maintains, display name
    /// ascending.
    fn scan_items(&self, addressbook_id: &str) -> Result<Vec<PimdirItem>> {
        let mut all: Vec<PimdirItem> = Vec::new();
        let mut cursor: Option<(String, i64)> = None;

        loop {
            let page = self.inner.reader.list_summaries(
                addressbook_id,
                cursor.as_ref().map(|(key, seq)| (key.as_str(), *seq)),
                SCAN_BATCH,
            )?;
            let len = page.len();
            if let Some(last) = page.last() {
                cursor = Some((last.sort_key.clone(), last.seq));
            }
            all.extend(page);
            if len < SCAN_BATCH {
                break;
            }
        }

        Ok(all)
    }

    /// Builds a shared [`Card`] from a stored item.
    ///
    /// The real body when it is local, else a preview projected from the
    /// stored summary, which keeps a listing useful on a partly synced
    /// store. The preview is never the record: [`get_card`](Self::get_card)
    /// refuses an unhydrated card outright.
    fn card_from_item(&self, addressbook_id: &str, item: PimdirItem) -> Result<Card> {
        let stored = match &item.object {
            Some(hash) => self.inner.blobs.get(hash)?,
            None => None,
        };

        let contents = match stored {
            Some(contents) => contents,
            None => {
                // NOTE: the blob may be gone from an inconsistent store
                // rather than never fetched, which is worth a word since
                // `get_card` refuses outright.
                if item.object.is_some() {
                    warn!(
                        "body blob missing for card `{}` in `{addressbook_id}`, \
                         listing its summary instead",
                        item.seq
                    );
                }

                preview_vcard(&item)
            }
        };

        Ok(Card {
            id: item.seq.to_string(),
            addressbook_id: addressbook_id.to_string(),
            etag: item.object.map(|hash| hash.0),
            contents,
        })
    }

    /// Fails unless the store knows `collection`, naming those it holds.
    ///
    /// The read seam answers an unknown collection with an empty page and
    /// the queue accepts any name, so without this a typo in `-k` would read
    /// as an empty addressbook and stage what nothing will ever apply. The
    /// ids carry the sync engine's namespace, so the refusal lists them.
    fn known_collection(&self, collection: &str) -> Result<()> {
        let mut ids: Vec<String> = self
            .collections()?
            .into_iter()
            .map(|candidate| candidate.id)
            .collect();

        if ids.iter().any(|id| id == collection) {
            return Ok(());
        }

        ids.sort();

        bail!(
            "Addressbook `{collection}` not found; this account holds: {}",
            ids.join(", "),
        )
    }

    /// The stored item behind a public card id, or a clear miss.
    fn item(&self, collection: &str, card_id: &str) -> Result<PimdirItem> {
        let seq = card_id
            .parse::<i64>()
            .map_err(|_| anyhow!("Invalid card id `{card_id}` (expected a number)"))?;

        self.inner
            .reader
            .get_item(collection, seq)?
            .ok_or_else(|| anyhow!("Card `{card_id}` not found in `{collection}`"))
    }

    /// Writes a body into the blob tree, then the action naming it.
    ///
    /// The body is durable before anything references it (pimdir STORAGE
    /// §14), and one producer wraps the pair rather than the enqueue alone:
    /// its shared lock is what keeps a collector out of the window between
    /// the two. A body the store already holds keeps the stored copy.
    fn stage(
        &mut self,
        collection: &str,
        contents: &[u8],
        action: impl FnOnce(PimdirHash) -> PimdirAction,
    ) -> Result<()> {
        let mut producer = self.inner.producer()?;

        // NOTE: the hash is the store's, read from `store_meta.hash_algo`:
        // a body named under another algorithm is one no read ever finds.
        let hash = producer.hash(contents);
        let mut writer = self.inner.blobs.writer()?;
        writer.write_all(contents)?;
        let size = writer.commit(&hash)?;
        let object = PimdirObject {
            hash,
            size: size as usize,
        };

        let action = action(object.hash.clone());
        let partials = producer
            .check(collection, &action)
            .map_err(|err| anyhow!("Stage the pimdir action: {err}"))?;
        producer
            .enqueue(collection, &action, Some(&object))
            .map_err(|err| anyhow!("Stage the pimdir action: {err}"))?;

        for partial in &partials {
            warn!("{partial}");
        }
        Ok(())
    }
}

/// How the refusal of a write whose `--if-match` names a version the
/// card no longer has starts, a prefix callers may match on.
pub const PRECONDITION_FAILED: &str = "Precondition failed";

/// Fails unless `if_match` names the card's current version.
///
/// The version is the store's hash of the card's body, the pending
/// queue folded in: the `etag` a read or a listing reports. A card whose
/// body is not local has no version to match. Surrounding double quotes
/// are dropped, so an HTTP-quoted tag matches as well.
fn check_version(
    item: &PimdirItem,
    addressbook_id: &str,
    card_id: &str,
    if_match: Option<&str>,
) -> Result<()> {
    let Some(expected) = if_match else {
        return Ok(());
    };
    let expected = expected.trim().trim_matches('"');
    let current = item.object.as_ref().map(|hash| hash.0.as_str());

    if current == Some(expected) {
        return Ok(());
    }

    Err(CodedError::new(
        ErrorCode::PreconditionFailed,
        format!(
            "{PRECONDITION_FAILED}: card `{card_id}` in addressbook `{addressbook_id}` is at \
             version `{}`, not `{expected}`; read it again",
            current.unwrap_or("none (body not fetched)"),
        ),
    )
    .into())
}

/// Renders a stored item's contact summary as the listing preview of a
/// card.
///
/// It carries only what the summary knows (`UID`, `FN`, `EMAIL`), which is
/// what makes a contact list readable before the bodies are synced. An
/// item holding no contact summary previews as a nameless card.
fn preview_vcard(item: &PimdirItem) -> Vec<u8> {
    let mut out = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\n");

    let Some(PimdirSummary::Contact(summary)) = &item.summary else {
        out.push_str("FN:\r\nEND:VCARD\r\n");
        return out.into_bytes();
    };

    if let Some(uid) = &summary.uid {
        // NOTE: two rows of one listing may legitimately carry this `UID`,
        // the store keying the second copy apart under a minted `dup:` link
        // id (pimdir STORAGE §9). It is a display value and never an
        // address, so nothing downstream may dedupe or group by it: the
        // public `seq` is what names a card.
        out.push_str(&format!("UID:{uid}\r\n"));
    }
    let full_name = summary
        .full_name
        .replace('\\', "\\\\")
        .replace(',', "\\,")
        .replace(';', "\\;")
        .replace('\n', "\\n");
    out.push_str(&format!("FN:{full_name}\r\n"));
    for email in &summary.emails {
        out.push_str(&format!("EMAIL:{}\r\n", email.address));
    }
    out.push_str("END:VCARD\r\n");

    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use io_pimdir::placement::{PimdirLevel, PimdirLinkId};

    use super::*;

    /// The address book the server names the default carries the mark, read
    /// from the role the sync engine recorded (pimdir STORAGE §14).
    #[test]
    fn the_default_address_book_is_the_one_its_store_marks() {
        let dir = tempfile::tempdir().unwrap();
        {
            let store = io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_account("work");
            for id in ["carddav/default", "carddav/team"] {
                store.ensure_collection(id, CARD_KIND).unwrap();
            }
            store
                .set_collection_role("carddav/default", Some("default"))
                .unwrap();
        }

        let mut backend = PimdirBackend::new(crate::config::PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        })
        .unwrap();
        let defaults: Vec<(String, bool)> = backend
            .list_addressbooks()
            .unwrap()
            .into_iter()
            .map(|book| (book.id, book.default))
            .collect();
        assert_eq!(
            defaults,
            [
                ("carddav/default".into(), true),
                ("carddav/team".into(), false)
            ]
        );
    }

    /// RFC 6352 §5.1 requires a `UID` unique per collection, which servers
    /// do not always enforce, most often after a repeated import.
    ///
    /// A store holds both copies, keying the second apart under a minted
    /// `dup:` link id (pimdir STORAGE §9). Both project as ordinary cards
    /// addressed by their `seq`, the minted key never reaching a reader.
    #[test]
    fn two_items_sharing_a_uid_project_two_distinct_cards() {
        let one = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:shared@example.org\r\n\
                    FN:Jane Doe\r\nEMAIL:jane@example.org\r\nEND:VCARD\r\n";
        let two = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:shared@example.org\r\n\
                    FN:Jane Doh\r\nEMAIL:doh@example.org\r\nEND:VCARD\r\n";

        // NOTE: a derivation is what a write carries, not a lookup: both
        // bodies derive the one bare link id, which is why the store mints.
        let first = contact::derive(one);
        let second = contact::derive(two);
        assert_eq!(first.link_id.0, "shared@example.org");
        assert_eq!(second.link_id.0, "shared@example.org");

        let bare = item(7, "shared@example.org", first.summary);
        let minted = item(
            8,
            "dup:shared@example.org#/books/contacts/copy.vcf",
            second.summary,
        );

        let previews = [&bare, &minted].map(|item| String::from_utf8(preview_vcard(item)).unwrap());

        // NOTE: both rows state the shared `UID` and neither is marked, so
        // it tells them apart from nothing.
        assert!(previews[0].contains("UID:shared@example.org\r\n"));
        assert!(previews[1].contains("UID:shared@example.org\r\n"));
        assert!(previews[0].contains("FN:Jane Doe\r\n"));
        assert!(previews[1].contains("FN:Jane Doh\r\n"));
        assert_ne!(previews[0], previews[1]);

        assert!(!previews[1].contains("dup:"));
    }

    /// A summary's unescaped name goes back out escaped, so the preview
    /// stays one content line per property.
    #[test]
    fn a_preview_escapes_the_name_the_summary_unescaped() {
        let card = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:c1\r\n\
                     FN:Doe\\, Jane\\; Jr.\r\nEND:VCARD\r\n";
        let preview = item(1, "c1", contact::derive(card).summary);

        let preview = String::from_utf8(preview_vcard(&preview)).unwrap();
        assert!(preview.contains("FN:Doe\\, Jane\\; Jr.\r\n"));
    }

    /// A stored item as a read hands one over, with no body fetched yet.
    fn item(seq: i64, link_id: &str, summary: Option<PimdirSummary>) -> PimdirItem {
        PimdirItem {
            seq,
            link_id: PimdirLinkId(link_id.to_string()),
            flags: PimdirFlags::default(),
            sort_key: String::new(),
            object: None,
            level: PimdirLevel::Meta,
            summary,
            retention: None,
        }
    }

    /// A store holding one address book, `book`, and the backend over it.
    fn store() -> (tempfile::TempDir, PimdirBackend) {
        let dir = tempfile::tempdir().unwrap();
        let store = io_pimdir::client::PimdirStore::open(dir.path()).unwrap();
        store.ensure_collection("book", CARD_KIND).unwrap();
        drop(store);

        let backend = PimdirBackend::new(PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        })
        .unwrap();

        (dir, backend)
    }

    /// Applies the queue as the store's owner, a sync, would.
    fn drain(dir: &tempfile::TempDir) {
        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("carddav")
            .drain()
            .unwrap();
    }

    /// A card created, applied, listed, updated and read keeps every
    /// field a contact list shows: names, addresses, numbers,
    /// organization, title and note.
    #[test]
    fn a_card_round_trips_its_common_fields_through_the_store() {
        use crate::shared::card::project::CardFields;

        let (dir, mut backend) = store();
        let card = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:c1\r\nFN:Jane Doe\r\n\
            N:Doe;Jane;;;\r\nEMAIL:jane@example.org\r\nEMAIL:doe@example.org\r\n\
            TEL:+33 1 00 00 00 00\r\nORG:Example Corp\r\nTITLE:Chief\r\n\
            NOTE:Likes tea\r\nEND:VCARD\r\n";

        let link = backend.create_card("book", card.to_vec()).unwrap();
        assert_eq!(link, "urn:uuid:c1");
        drain(&dir);

        let cards = backend.list_cards("book", None, None).unwrap();
        assert_eq!(cards.len(), 1);
        let fields = CardFields::project(&cards[0].contents);
        assert_eq!(fields.full_name.as_deref(), Some("Jane Doe"));
        assert_eq!(fields.emails, ["jane@example.org", "doe@example.org"]);
        assert_eq!(fields.phones, ["+33 1 00 00 00 00"]);
        assert_eq!(fields.organization.as_deref(), Some("Example Corp"));
        assert_eq!(fields.title.as_deref(), Some("Chief"));
        assert_eq!(fields.note.as_deref(), Some("Likes tea"));

        let id = cards[0].id.clone();
        let updated = String::from_utf8(card.to_vec())
            .unwrap()
            .replace("TITLE:Chief", "TITLE:Director");
        backend
            .update_card("book", &id, updated.into_bytes(), None)
            .unwrap();

        // NOTE: the reader folds the pending update over the stored row,
        // so the change reads back before a sync applies it.
        let read = backend.get_card("book", &id).unwrap();
        assert_eq!(
            CardFields::project(&read.contents).title.as_deref(),
            Some("Director")
        );
    }

    const JANE: &[u8] = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:c1\r\n\
                          FN:Jane Doe\r\nEMAIL:jane@example.org\r\nEND:VCARD\r\n";

    /// How many actions wait in the queue of `book`.
    fn pending(backend: &PimdirBackend) -> usize {
        backend
            .inner
            .producer()
            .unwrap()
            .pending_actions("book")
            .unwrap()
            .len()
    }

    /// A read and a listing report one version, the body's hash, and a
    /// staged update gated on it moves it at once.
    #[test]
    fn the_version_is_the_body_hash_and_follows_a_staged_update() {
        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drain(&dir);

        let listed = backend.list_cards("book", None, None).unwrap();
        let id = listed[0].id.clone();
        let etag = backend.get_card("book", &id).unwrap().etag.unwrap();
        assert_eq!(listed[0].etag.as_deref(), Some(etag.as_str()));

        let edited = String::from_utf8(JANE.to_vec())
            .unwrap()
            .replace("Jane Doe", "Jane Roe");
        backend
            .update_card("book", &id, edited.into_bytes(), Some(&etag))
            .unwrap();

        let moved = backend.get_card("book", &id).unwrap().etag.unwrap();
        assert_ne!(moved, etag);
    }

    /// A write naming a version the card no longer has queues nothing and
    /// says so with the stable prefix; the right one deletes it.
    /// A stale version and an undownloaded body carry their stable code.
    #[test]
    fn a_stale_if_match_carries_its_code() {
        use crate::error::{ErrorCode, code_of};

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drain(&dir);
        let id = backend.list_cards("book", None, None).unwrap()[0]
            .id
            .clone();

        let err = backend.delete_card("book", &id, Some("stale")).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::PreconditionFailed));
    }

    #[test]
    fn a_stale_if_match_refuses_the_update_and_the_delete() {
        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drain(&dir);
        let id = backend.list_cards("book", None, None).unwrap()[0]
            .id
            .clone();

        let update = backend.update_card("book", &id, JANE.to_vec(), Some("stale"));
        let err = update.unwrap_err().to_string();
        assert!(err.starts_with(PRECONDITION_FAILED), "{err}");
        assert!(err.contains("`stale`"), "{err}");

        let delete = backend.delete_card("book", &id, Some("stale"));
        let err = delete.unwrap_err().to_string();
        assert!(err.starts_with(PRECONDITION_FAILED), "{err}");
        assert_eq!(pending(&backend), 0);

        let etag = backend.get_card("book", &id).unwrap().etag.unwrap();
        backend
            .delete_card("book", &id, Some(&format!("\"{etag}\"")))
            .unwrap();
        assert_eq!(pending(&backend), 1);
        assert!(backend.list_cards("book", None, None).unwrap().is_empty());

        drain(&dir);
        assert!(backend.get_card("book", &id).is_err());
    }
}
