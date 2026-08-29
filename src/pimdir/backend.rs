//! # Pimdir backend
//!
//! Pimdir arm of the shared-API client, mapping the shared addressbook
//! and card operations onto a local pimdir store.
//!
//! Reads build cards from the stored `v: 1` summary (pimdir SPEC Annex A)
//! plus the blob store: a card whose body is not local still lists, and
//! reading it reports "body not fetched" rather than erroring.
//!
//! Writes append one action to the store's queue (pimdir SPEC §15.1)
//! through a producer opened for that write, the body first, then the row
//! pinning it. The store's owner, a sync, applies and pushes it, and the
//! reader folds the pending queue so a staged change shows here at once.

use std::io::Write;

use anyhow::{Result, anyhow, bail};
use io_pimdir::{
    PimdirCollection, PimdirItem,
    codec::PimdirAction,
    conventions::card::{self, PimdirCardMeta},
};
use io_replica::{object::ReplicaHash, placement::ReplicaFlags};
use log::warn;

use crate::{
    config::PimdirConfig,
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
            })
            .collect();

        addressbooks.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(addressbooks)
    }

    /// Always fails, a collection being the store owner's to declare.
    ///
    /// This backend is a producer (pimdir SPEC §8), which appends item
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
            bail!(
                "Card `{card_id}` in `{addressbook_id}` is not downloaded yet \
                 (body not fetched); run a sync to hydrate it"
            );
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
    /// store-assigned id to report yet.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        self.known_collection(addressbook_id)?;

        let derived = card::derive(&contents);
        let link_id = derived.link_id.clone();

        self.stage(addressbook_id, &contents, |hash| PimdirAction::Add {
            link_id: Some(derived.link_id),
            flags: ReplicaFlags::default(),
            object: Some(hash),
            meta: Some(derived.meta),
            handle: None,
        })?;

        Ok(link_id.0)
    }

    /// Stages a body replacement for `card_id` as an `update` action.
    ///
    /// The next sync applies and pushes it, three-way merging against the
    /// stored base. `if_match` is ignored: reconciling against that base is
    /// stronger than an ETag precondition a local store cannot check.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        _if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        self.known_collection(addressbook_id)?;

        let seq = self.item(addressbook_id, card_id)?.seq;
        let derived = card::derive(&contents);

        self.stage(addressbook_id, &contents, |hash| PimdirAction::Update {
            seq,
            object: hash,
            meta: Some(derived.meta),
        })?;

        Ok(CardUpdateOutcome::default())
    }

    /// Stages a `remove` action for `card_id`.
    ///
    /// The next sync applies it as a tombstone and pushes a server-side
    /// delete.
    pub fn delete_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<()> {
        self.known_collection(addressbook_id)?;

        let seq = self.item(addressbook_id, card_id)?.seq;
        self.inner
            .producer()?
            .enqueue(addressbook_id, &PimdirAction::Remove { seq }, None, &now())
            .map_err(|err| anyhow!("Stage the pimdir action: {err}"))?;

        Ok(())
    }

    /// The address book collections of the configured account.
    ///
    /// One store holds every kind a sync caches (pimdir SPEC §9.2), so the
    /// kind separates an address book from a mailbox or a calendar. A
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

    /// Pulls every live item of a collection by keyset paging.
    ///
    /// The order is the contacts one the store maintains, display name
    /// ascending.
    fn scan_items(&self, addressbook_id: &str) -> Result<Vec<PimdirItem>> {
        let mut all: Vec<PimdirItem> = Vec::new();
        let mut cursor: Option<(String, i64)> = None;

        loop {
            let page = self.inner.reader.list_items_page_asc(
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
    /// `v: 1` summary, which keeps a listing useful on a partly synced
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

                preview_vcard(&summary_of(&item))
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
    /// The body is durable before anything references it (pimdir SPEC §14),
    /// and one producer wraps the pair rather than the enqueue alone: its
    /// shared lock is what keeps a collector out of the window between the
    /// two. A body the store already holds keeps the stored copy.
    fn stage(
        &self,
        collection: &str,
        contents: &[u8],
        action: impl FnOnce(ReplicaHash) -> PimdirAction,
    ) -> Result<()> {
        let mut producer = self.inner.producer()?;

        // NOTE: the hash is the store's, read from `store_meta.hash_algo`:
        // a body named under another algorithm is one no read ever finds.
        let hash = producer.hash(contents);
        let mut writer = self.inner.blobs.writer()?;
        writer.write_all(contents)?;
        let size = writer.commit(&hash)?;

        producer
            .enqueue(collection, &action(hash), Some(size), &now())
            .map_err(|err| anyhow!("Stage the pimdir action: {err}"))?;

        Ok(())
    }
}

/// The enqueue timestamp, RFC 3339 as the queue column expects.
fn now() -> String {
    humantime::format_rfc3339_millis(std::time::SystemTime::now()).to_string()
}

/// Reads a stored item's `v: 1` summary.
///
/// Falls back to an empty one when the card was never projected or its
/// summary does not parse.
fn summary_of(item: &PimdirItem) -> PimdirCardMeta {
    item.meta
        .as_ref()
        .and_then(|meta| serde_json::from_str(&meta.0).ok())
        .unwrap_or_default()
}

/// Renders a stored summary as the listing preview of a card.
///
/// It carries only what the summary knows (`UID`, `FN`, `EMAIL`), which is
/// what makes a contact list readable before the bodies are synced.
fn preview_vcard(summary: &PimdirCardMeta) -> Vec<u8> {
    let mut out = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\n");

    if let Some(uid) = &summary.uid {
        // NOTE: two rows of one listing may legitimately carry this `UID`,
        // the store keying the second copy apart under a minted `dup:` link
        // id (pimdir SPEC §9). It is a display value and never an address,
        // so nothing downstream may dedupe or group by it: the public `seq`
        // is what names a card.
        out.push_str(&format!("UID:{uid}\r\n"));
    }
    out.push_str(&format!("FN:{}\r\n", summary.fn_));
    for email in &summary.emails {
        out.push_str(&format!("EMAIL:{email}\r\n"));
    }
    out.push_str("END:VCARD\r\n");

    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use io_replica::placement::{ReplicaLevel, ReplicaLinkId, ReplicaMeta};

    use super::*;

    /// RFC 6352 §5.1 requires a `UID` unique per collection, which servers
    /// do not always enforce, most often after a repeated import.
    ///
    /// A store holds both copies, keying the second apart under a minted
    /// `dup:` link id (pimdir SPEC §9). Both project as ordinary cards
    /// addressed by their `seq`, the minted key never reaching a reader.
    #[test]
    fn two_items_sharing_a_uid_project_two_distinct_cards() {
        let one = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:shared@example.org\r\n\
                    FN:Jane Doe\r\nEMAIL:jane@example.org\r\nEND:VCARD\r\n";
        let two = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:shared@example.org\r\n\
                    FN:Jane Doh\r\nEMAIL:doh@example.org\r\nEND:VCARD\r\n";

        // NOTE: a derivation is what a write carries, not a lookup: both
        // bodies derive the one bare link id, which is why the store mints.
        let first = card::derive(one);
        let second = card::derive(two);
        assert_eq!(first.link_id.0, "shared@example.org");
        assert_eq!(second.link_id.0, "shared@example.org");

        let bare = item(7, "shared@example.org", first.meta);
        let minted = item(
            8,
            "dup:shared@example.org#/books/contacts/copy.vcf",
            second.meta,
        );

        let previews = [&bare, &minted]
            .map(|item| String::from_utf8(preview_vcard(&summary_of(item))).unwrap());

        // NOTE: both rows state the shared `UID` and neither is marked, so
        // it tells them apart from nothing.
        assert!(previews[0].contains("UID:shared@example.org\r\n"));
        assert!(previews[1].contains("UID:shared@example.org\r\n"));
        assert!(previews[0].contains("FN:Jane Doe\r\n"));
        assert!(previews[1].contains("FN:Jane Doh\r\n"));
        assert_ne!(previews[0], previews[1]);

        assert!(!previews[1].contains("dup:"));
    }

    /// A stored item as a read hands one over, with no body fetched yet.
    fn item(seq: i64, link_id: &str, meta: ReplicaMeta) -> PimdirItem {
        PimdirItem {
            seq,
            link_id: ReplicaLinkId(link_id.to_string()),
            flags: ReplicaFlags::default(),
            meta: Some(meta),
            sort_key: String::new(),
            object: None,
            level: ReplicaLevel::Meta,
            retention: None,
        }
    }
}
