//! pimdir arm of the shared-API client: glue mapping the shared
//! addressbook and card operations onto a local pimdir store.
//!
//! Reads build cards from the stored `v: 1` summary (pimdir SPEC Annex
//! A) plus the blob store. A card whose body is not local still lists,
//! and reading it reports "body not fetched" rather than erroring.
//!
//! Writes append one action to the store's queue (pimdir SPEC §15.1)
//! through a producer opened for that write: the body reaches the blob
//! tree first, then the row pinning it. The store's owner, a sync,
//! applies the action and pushes it. The same reader folds the pending
//! queue over its reads, so a staged change shows here before that
//! happens.

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

/// pimdir backend of the shared-API client, wrapping an opened local store.
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

    /// Lists the contact collections (declared `text/vcard`, or kind-less
    /// legacy ones), sorted by name.
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

    /// Always fails: declaring a collection is an owner write (pimdir SPEC
    /// §8) and this backend is a producer, which appends item actions and
    /// nothing else. A collection declared here would also be one no sync
    /// knows about, so no address book would ever be created from it.
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

    /// Always fails: a collection's row (id, display name, description,
    /// colour) is what a sync writes from the server, and this backend stages
    /// item actions only, so every field of it is refused rather than changed
    /// locally into something no sync would carry.
    ///
    /// A rename is refused for a sharper reason too: io-pimdir's
    /// `rename_collection` renames the *identifier*, so honouring `--name`
    /// here would move every card under an id nobody asked for.
    pub fn update_addressbook(&mut self, _id: &str, _patch: AddressbookDiff) -> Result<()> {
        bail!(
            "The pimdir backend cannot update an addressbook: its name, description \
             and color come from the server through a sync. Rename it there and sync"
        )
    }

    /// Always fails: io-pimdir exposes no collection removal, and the queue
    /// carries item actions only, so a delete here would be a silent no-op.
    pub fn delete_addressbook(&mut self, _id: &str) -> Result<()> {
        bail!(
            "The pimdir backend cannot delete an addressbook; \
             delete it on the server and sync, or remove the store directory"
        )
    }

    /// Lists the cards inside `addressbook_id` in the store's own contacts
    /// order (display name ascending), applying 1-indexed pagination.
    ///
    /// An unhydrated card lists as a preview projected from its stored
    /// summary; [`get_card`](Self::get_card) is where the absence is
    /// reported.
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

    /// Fetches `card_id` from `addressbook_id`, reading its content-addressed
    /// blob. Fails with a clear "body not fetched" when the card is not
    /// hydrated to `Full` (no local body), the cue to sync rather than a
    /// data-loss error.
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

    /// Stages a body replacement for `card_id` as an `update` action the next
    /// sync applies and pushes, three-way merging against the stored base.
    ///
    /// `if_match` is ignored: the applied edit is reconciled by the engine
    /// against the base body it recorded at sync time, which is a stronger
    /// guarantee than an ETag precondition a local store cannot check.
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

    /// Stages a `remove` action for `card_id`, which the next sync applies as
    /// a tombstone and pushes as a server-side delete.
    pub fn delete_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<()> {
        self.known_collection(addressbook_id)?;

        let seq = self.item(addressbook_id, card_id)?.seq;
        self.inner
            .producer()?
            .enqueue(addressbook_id, &PimdirAction::Remove { seq }, None, &now())
            .map_err(|err| anyhow!("Stage the pimdir action: {err}"))?;

        Ok(())
    }

    /// The store's address book collections, narrowed to the configured
    /// account when the store groups several (pimdir SPEC §9.2).
    ///
    /// One store holds the collections of every kind a sync caches, so the
    /// kind is what separates an address book from a mailbox or a calendar.
    /// A kind-less collection counts: a sync that created one before kinds
    /// were declared left the column empty.
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

    /// Pulls every live item of a collection by keyset paging, in the
    /// contacts order the store maintains (display name ascending).
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

    /// Builds a shared [`Card`] from a stored item: the real body when it is
    /// local, else a preview synthesized from the stored `v: 1` summary, so a
    /// listing stays useful on a partially synced store.
    ///
    /// NOTE: the preview is a projection of the summary, never the document of
    /// record. [`get_card`](Self::get_card) refuses an unhydrated card outright
    /// rather than handing the preview back, so nothing downstream can mistake
    /// one for a real card.
    fn card_from_item(&self, addressbook_id: &str, item: PimdirItem) -> Result<Card> {
        let stored = match &item.object {
            Some(hash) => self.inner.blobs.get(hash)?,
            None => None,
        };

        let contents = match stored {
            Some(contents) => contents,
            None => {
                // NOTE: no body to show, either because the card was never
                // hydrated or because its blob is gone from an inconsistent
                // store. The preview keeps the row readable; the second case
                // is worth a word, since `get_card` will refuse outright.
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

    /// Fails unless `collection` is a collection the store knows, naming the
    /// ones it does hold.
    ///
    /// The store's read seam answers an unknown collection with an empty page
    /// and its queue accepts an action for any name, so without this a typo in
    /// `-k` would read as an empty addressbook and stage into one nothing will
    /// ever apply. An addressbook is its collection id, which carries the sync
    /// engine's namespace and is not guessable, so the refusal shows the ids to
    /// choose from.
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

    /// Writes a body into the blob tree, then appends the action naming it,
    /// both under one producer.
    ///
    /// The body is durable before anything references it (pimdir SPEC §14),
    /// and the producer is opened around the pair rather than for the enqueue
    /// alone: its shared lock is what keeps a collector out of the window
    /// between a body reaching the blob tree and the queue row pinning it. A
    /// body the store already holds keeps the stored copy.
    fn stage(
        &self,
        collection: &str,
        contents: &[u8],
        action: impl FnOnce(ReplicaHash) -> PimdirAction,
    ) -> Result<()> {
        let mut producer = self.inner.producer()?;

        // NOTE: the hash is the store's, read from `store_meta.hash_algo`,
        // never one this crate picks: a body named under another algorithm
        // is a body no read ever finds.
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

/// Reads a stored item's `v: 1` summary, falling back to an empty one when the
/// card was never projected or the blob does not parse.
fn summary_of(item: &PimdirItem) -> PimdirCardMeta {
    item.meta
        .as_ref()
        .and_then(|meta| serde_json::from_str(&meta.0).ok())
        .unwrap_or_default()
}

/// Renders a stored summary as a minimal vCard, the listing preview of a card
/// whose body is not local yet. It carries only what the summary knows (`UID`,
/// `FN`, `EMAIL`), so a contact list reads correctly before a full sync.
fn preview_vcard(summary: &PimdirCardMeta) -> Vec<u8> {
    let mut out = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\n");

    if let Some(uid) = &summary.uid {
        // NOTE: two rows of one listing may legitimately carry this `UID`,
        // the store keying the second copy apart under a minted `dup:` link
        // id (pimdir SPEC §9). It is a display value and never an address,
        // so nothing downstream may dedupe, group or look a row up by it:
        // the public `seq` is what names a card.
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

    /// RFC 6352 §5.1 requires a card's `UID` to be unique in its collection,
    /// and servers do not always enforce it, most often after a repeated
    /// import. A store therefore holds both copies, keying the second apart
    /// under a minted `dup:` link id (pimdir SPEC §9), and both project as
    /// ordinary cards: the shared `UID` tells them apart from nothing, so
    /// what addresses them is the public `seq` each carries, and the key the
    /// store minted never reaches the card a reader sees.
    #[test]
    fn two_items_sharing_a_uid_project_two_distinct_cards() {
        let one = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:shared@example.org\r\n\
                    FN:Jane Doe\r\nEMAIL:jane@example.org\r\nEND:VCARD\r\n";
        let two = b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:shared@example.org\r\n\
                    FN:Jane Doh\r\nEMAIL:doh@example.org\r\nEND:VCARD\r\n";

        // A derivation is what a write carries, not a lookup: both bodies
        // derive the one bare link id, which is why the store mints.
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

        // Both rows state the shared `UID` and neither is marked, so it
        // tells them apart from nothing, and the two cards stay distinct.
        assert!(previews[0].contains("UID:shared@example.org\r\n"));
        assert!(previews[1].contains("UID:shared@example.org\r\n"));
        assert!(previews[0].contains("FN:Jane Doe\r\n"));
        assert!(previews[1].contains("FN:Jane Doh\r\n"));
        assert_ne!(previews[0], previews[1]);

        // The key the store minted is its own, and never reaches a reader.
        assert!(!previews[1].contains("dup:"));
    }

    /// A stored item as a read hands one over: the public `seq`, the key the
    /// store assigned it, and the `v: 1` summary a sync projected, with no
    /// body fetched yet.
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
