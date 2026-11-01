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
//! through a producer opened for that write, the body and the row pinning
//! it in one step, nothing written when the producer refuses the action.
//! An edit made against a body it read names that body, so the owner
//! parks it rather than overwrite a change a sync brought in since. The
//! owner applies and pushes the rest, and the reader folds the pending
//! queue so a staged update or removal shows here at once. A staged
//! create lists once the owner applies it, having no public id before.

use std::ops::ControlFlow;

use anyhow::{Result, anyhow, bail};
use io_pimdir::{
    client::reader::{PimdirCollection, PimdirItem},
    codec::PimdirAction,
    object::PimdirHash,
    placement::PimdirFlags,
    summary::{PimdirSummary, contact},
};
use log::warn;
use pimalaya_cli::error::CodedError;

use crate::{
    config::{AccountConfig, PimdirConfig},
    error::ErrorCode,
    pimdir::{
        client::{PimdirClient, refusal},
        status::{PimdirAddressbookStatus, PimdirParkedStatus, PimdirStatusOutput},
    },
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
    },
};

/// The media type a pimdir collection carries to be an addressbook.
const CARD_KIND: &str = "text/vcard";

/// How many items to pull per keyset page when scanning a whole collection.
#[cfg(not(test))]
const SCAN_BATCH: usize = 500;

/// A batch small enough for a test store to span several.
#[cfg(test)]
const SCAN_BATCH: usize = 7;

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

    /// Opens the store from the `[pimdir]` block of the account `name`,
    /// as a `pimdir` command needs it.
    ///
    /// Bails when the account declares no such block.
    pub fn build(name: String, mut account_config: AccountConfig) -> Result<Self> {
        let config = account_config
            .pimdir
            .take()
            .ok_or_else(|| anyhow!("Pimdir config is missing for account `{name}`"))?;
        Self::new(config)
    }

    /// Exercises both roles this backend takes: lists the address books
    /// through the reader, then opens a producer and probes it, so a store
    /// whose staging lock or write lock is out of reach fails here rather
    /// than on the first write.
    ///
    /// The probe takes and releases both locks without waiting on a
    /// collector and writes nothing into the store.
    pub fn check(&mut self) -> Result<()> {
        self.list_addressbooks()?;
        self.inner
            .producer()?
            .probe()
            .map_err(|err| refusal(err, "Probe the pimdir store as a producer"))
    }

    /// Reports what the store holds for the account read: its address
    /// books, how much of each is downloaded and queued, and the actions
    /// the store's owner parked in them.
    ///
    /// A parked action is one the owner will never apply (pimdir STORAGE
    /// §15.3), so the write a command reported as queued is lost unless
    /// something shows it. Narrowed to the account's address books, as
    /// every other read is.
    pub fn status(&mut self) -> Result<PimdirStatusOutput> {
        let accounts = self.inner.reader.list_accounts()?;
        let mut addressbooks = Vec::new();

        for collection in self.collections()? {
            let (mut total, mut hydrated) = (0, 0);
            self.scan(&collection.id, |item| {
                total += 1;
                hydrated += usize::from(item.object.is_some());
                Ok(ControlFlow::Continue(()))
            })?;
            let queued = self.inner.reader.count_pending_creates(&collection.id)?;

            addressbooks.push(PimdirAddressbookStatus {
                name: if collection.name.is_empty() {
                    collection.id.clone()
                } else {
                    collection.name
                },
                id: collection.id,
                total,
                hydrated,
                queued,
            });
        }

        addressbooks.sort_by(|a, b| a.name.cmp(&b.name));

        let parked = self
            .inner
            .reader
            .parked_actions()?
            .into_iter()
            .filter(|parked| {
                addressbooks
                    .iter()
                    .any(|addressbook| addressbook.id == parked.collection)
            })
            .map(|parked| PimdirParkedStatus {
                row: parked.id,
                addressbook_id: parked.collection,
                action: parked.action,
                error: parked.error,
            })
            .collect();

        Ok(PimdirStatusOutput {
            account: self.inner.account.clone(),
            accounts,
            addressbooks,
            parked,
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
             delete it on the server and sync"
        )
    }

    /// Lists the cards inside `addressbook_id`, 1-indexed paging.
    ///
    /// The order is the store's own contacts one, display name ascending.
    /// An unhydrated card lists as a preview projected from its stored
    /// summary, [`get_card`](Self::get_card) reporting the absence.
    ///
    /// The collection streams through in keyset batches: the cards before
    /// the page are counted off their rows, a body is read only for a card
    /// the page returns, and the scan stops once the page is full.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        self.known_collection(addressbook_id)?;

        let (mut skip, take) = match page_size {
            None => (0, None),
            Some(0) => return Ok(Vec::new()),
            Some(size) => {
                let page = page.unwrap_or(1).max(1);
                let skip = ((page - 1) as usize).saturating_mul(size as usize);
                (skip, Some(size as usize))
            }
        };
        let mut cards = Vec::new();

        self.scan(addressbook_id, |item| {
            if skip > 0 {
                skip -= 1;
                return Ok(ControlFlow::Continue(()));
            }

            cards.push(self.card_from_item(addressbook_id, item)?);

            match take {
                Some(take) if cards.len() >= take => Ok(ControlFlow::Break(())),
                _ => Ok(ControlFlow::Continue(())),
            }
        })?;

        Ok(cards)
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
            self.inner.reader.blobs().get(&hash)?.ok_or_else(|| {
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
        self.writable(addressbook_id)?;

        let link_id = contact::derive(&contents).link_id;

        self.stage(addressbook_id, &contents, |hash| PimdirAction::Add {
            link_id: Some(link_id.clone()),
            flags: PimdirFlags::default(),
            object: Some(hash),
            supersedes: None,
        })?;

        Ok(link_id.0)
    }

    /// Stages a body replacement for `card_id` as an `update` action.
    ///
    /// The next sync applies and pushes it, three-way merging against the
    /// stored base. `if_match` gates the staging on the card's version,
    /// the store's hash of its body with the pending queue folded in: the
    /// gate is the caller's, the merge the store's.
    ///
    /// The action names the body the edit was made against, the one
    /// `if_match` matched (pimdir STORAGE §15.3), so the owner parks it
    /// when a sync replaced that body before the drain. With no `if_match`
    /// the caller read nothing, and the action names none.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        self.writable(addressbook_id)?;

        let item = self.item(addressbook_id, card_id)?;
        check_version(self.version(&item)?, addressbook_id, card_id, if_match)?;
        let seq = item.seq;
        // NOTE: the version matched, so the item's body is the one read.
        let read = if_match.and(item.object);

        self.stage(addressbook_id, &contents, |hash| PimdirAction::Update {
            seq,
            object: hash,
            if_match: read,
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
        self.writable(addressbook_id)?;

        let item = self.item(addressbook_id, card_id)?;
        check_version(self.version(&item)?, addressbook_id, card_id, if_match)?;
        let seq = item.seq;
        let action = PimdirAction::Remove { seq };
        let mut producer = self.inner.producer()?;
        let partials = producer
            .check(addressbook_id, &action)
            .map_err(|err| refusal(err, "Stage the pimdir action"))?;
        producer
            .enqueue(addressbook_id, &action, None)
            .map_err(|err| refusal(err, "Stage the pimdir action"))?;

        for partial in &partials {
            warn!("{partial}");
        }
        Ok(())
    }

    /// The address book collections of the configured account.
    ///
    /// One store holds every kind a sync caches (pimdir STORAGE §4.3, §14),
    /// so the kind separates an address book from a mailbox or a calendar.
    /// A kind-less one counts: a sync predating kinds left the column empty.
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
            .filter(|collection| is_card_kind(&collection.kind))
            .collect())
    }

    /// Walks every live item of a collection by keyset paging, each with
    /// its summary joined, until `visit` breaks.
    ///
    /// The order is the contacts one the store maintains, display name
    /// ascending. One batch of rows is held at a time, never the
    /// collection.
    fn scan(
        &self,
        addressbook_id: &str,
        mut visit: impl FnMut(PimdirItem) -> Result<ControlFlow<()>>,
    ) -> Result<()> {
        let mut cursor: Option<(String, i64)> = None;

        loop {
            let batch = self.inner.reader.list_summaries(
                addressbook_id,
                cursor.as_ref().map(|(key, seq)| (key.as_str(), *seq)),
                SCAN_BATCH,
            )?;
            let len = batch.len();
            if let Some(last) = batch.last() {
                cursor = Some((last.sort_key.clone(), last.seq));
            }
            for item in batch {
                if visit(item)?.is_break() {
                    return Ok(());
                }
            }
            if len < SCAN_BATCH {
                return Ok(());
            }
        }
    }

    /// Builds a shared [`Card`] from a stored item.
    ///
    /// The real body when it is local, else a preview projected from the
    /// stored summary, which keeps a listing useful on a partly synced
    /// store. The preview is never the record: [`get_card`](Self::get_card)
    /// refuses an unhydrated card outright.
    fn card_from_item(&self, addressbook_id: &str, item: PimdirItem) -> Result<Card> {
        let stored = match &item.object {
            Some(hash) => self.inner.reader.blobs().get(hash)?,
            None => None,
        };
        // NOTE: the version is the body's, so a card whose blob is gone
        // has none, as an unfetched one has none.
        let etag = match stored {
            Some(_) => item.object.as_ref().map(|hash| hash.0.clone()),
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
            etag,
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

    /// Fails unless `collection` is an address book of the account that a
    /// source syncs, the one place a staged write ever leaves the store.
    ///
    /// The owner applies a queued action as a source syncing its
    /// collection, and with none it binds whichever source drains first to
    /// a collection that source never lists: the card would never be
    /// pushed. A collection the device keeps for itself is the owner's to
    /// write (pimdir STORAGE §14.3), never a producer's.
    fn writable(&self, collection: &str) -> Result<()> {
        self.known_collection(collection)?;

        if self.inner.reader.collection_sources(collection)?.is_empty() {
            bail!(
                "Addressbook `{collection}` is one no source syncs, so a card written \
                 there would never reach a server; write into a synced addressbook"
            );
        }

        Ok(())
    }

    /// A stored item's version, the hash of its body, when the body is
    /// local: none for one never fetched and none for one whose blob is
    /// gone from the store.
    fn version(&self, item: &PimdirItem) -> Result<Option<String>> {
        let Some(hash) = &item.object else {
            return Ok(None);
        };

        let local = self.inner.reader.blobs().reader(hash)?.is_some();
        Ok(local.then(|| hash.0.clone()))
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

    /// Stages a body and the action naming it as one step.
    ///
    /// `PimdirProducer::enqueue_with_body_checked` checks the action
    /// first: the gate of pimdir STORAGE §15.6 and, for an `add`, a link id
    /// the address book holds live or has queued, which the owner would
    /// park at drain (§15.3). Only then does it write the body under its
    /// staging lock, named by the store's own hash, and append the row
    /// pinning it, in one transaction: a refused write leaves no body in
    /// the blob tree, and a collector never sees one unpinned. The partial
    /// supports the gate passed the action with are logged.
    fn stage(
        &mut self,
        collection: &str,
        contents: &[u8],
        action: impl FnOnce(PimdirHash) -> PimdirAction,
    ) -> Result<()> {
        let enqueued = self
            .inner
            .producer()?
            .enqueue_with_body_checked(collection, contents, action)
            .map_err(|err| refusal(err, "Stage the pimdir action"))?;

        for partial in &enqueued.partial {
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
    current: Option<String>,
    addressbook_id: &str,
    card_id: &str,
    if_match: Option<&str>,
) -> Result<()> {
    let Some(expected) = if_match else {
        return Ok(());
    };
    let expected = expected.trim().trim_matches('"');

    if current.as_deref() == Some(expected) {
        return Ok(());
    }

    Err(CodedError::new(
        ErrorCode::PreconditionFailed,
        format!(
            "{PRECONDITION_FAILED}: card `{card_id}` in addressbook `{addressbook_id}` is at \
             version `{}`, not `{expected}`; read it again",
            current.as_deref().unwrap_or("none (body not local)"),
        ),
    )
    .into())
}

/// Whether a collection kind makes an address book: its media type, the
/// part before any `;` parameter, is `text/vcard`, as io-pimdir reads it
/// when it picks the summaries, or it is empty, a sync predating kinds
/// having left it so.
fn is_card_kind(kind: &str) -> bool {
    kind.is_empty() || kind.split(';').next().unwrap_or_default().trim() == CARD_KIND
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

    /// Declares `collection` an address book the source `carddav` syncs,
    /// as the first round of a sync leaves it: a checkpoint, no member yet,
    /// grouped under `account` when one is named.
    fn synced(root: &std::path::Path, account: Option<&str>, collection: &str) {
        use io_pimdir::{
            change::PimdirWriteOp,
            collection::{PimdirCheckpoint, PimdirCollectionId},
        };

        let mut store = io_pimdir::client::PimdirStore::open(root)
            .unwrap()
            .for_source("carddav");
        if let Some(account) = account {
            store = store.for_account(account);
        }
        store.ensure_collection(collection, CARD_KIND).unwrap();
        store
            .write(vec![PimdirWriteOp::SetCheckpoint {
                collection: PimdirCollectionId::from(collection),
                checkpoint: PimdirCheckpoint(Vec::new()),
            }])
            .unwrap();
    }

    /// A store holding one synced address book, `book`, and the backend
    /// over it.
    fn store() -> (tempfile::TempDir, PimdirBackend) {
        let dir = tempfile::tempdir().unwrap();
        synced(dir.path(), None, "book");

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
        use pimalaya_cli::error::code_of;

        use crate::error::ErrorCode;

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

    /// A card named `name`, its `UID` and address derived from `index`.
    fn card(index: usize, name: &str) -> Vec<u8> {
        format!(
            "BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:card-{index}\r\nFN:{name}\r\n\
             EMAIL:card-{index}@example.org\r\nEND:VCARD\r\n"
        )
        .into_bytes()
    }

    /// An address book of `count` cards whose names sort apart from
    /// their creation order, some sharing one name, applied by a sync.
    fn book(count: usize) -> (tempfile::TempDir, PimdirBackend) {
        let (dir, mut backend) = store();
        let names = ["Zoé", "adam", "Émile", "bob", "Bob", "", "carla", "Zoé"];
        for index in 0..count {
            let name = format!("{} {index:03}", names[index % names.len()]);
            backend
                .create_card("book", card(index, name.trim()))
                .unwrap();
        }
        drain(&dir);
        (dir, backend)
    }

    /// The listing as it was before it streamed: every summary, then
    /// every card built with its body, then the page cut in memory.
    fn eager(
        backend: &PimdirBackend,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        let mut all = Vec::new();
        backend.scan("book", |item| {
            all.push(item);
            Ok(ControlFlow::Continue(()))
        })?;

        let cards = all
            .into_iter()
            .map(|item| backend.card_from_item("book", item))
            .collect::<Result<Vec<_>>>()?;

        Ok(crate::shared::client::paginate(cards, page, page_size))
    }

    /// Every page comes out as it did when the whole address book was
    /// read first: same cards, same order, same bodies and versions.
    fn assert_streams_as_it_listed(backend: &mut PimdirBackend) {
        let mut listed = 0;
        for page_size in [
            None,
            Some(0),
            Some(1),
            Some(3),
            Some(7),
            Some(10),
            Some(100),
        ] {
            for page in [None, Some(0), Some(1), Some(2), Some(5), Some(9), Some(50)] {
                let streamed = backend.list_cards("book", page, page_size).unwrap();
                let expected = eager(backend, page, page_size).unwrap();
                assert_eq!(streamed, expected, "page {page:?} of {page_size:?}");
                listed += streamed.len();
            }
        }
        assert!(listed > 300, "{listed}");
    }

    #[test]
    fn a_streamed_listing_lists_what_the_whole_read_listed() {
        let (_dir, mut backend) = book(40);
        assert_streams_as_it_listed(&mut backend);

        // NOTE: the pending queue folds in as it did, a staged update
        // and a staged removal included.
        let ids: Vec<String> = backend
            .list_cards("book", None, None)
            .unwrap()
            .into_iter()
            .map(|card| card.id)
            .collect();
        backend
            .update_card("book", &ids[3], card(3, "Renamed"), None)
            .unwrap();
        assert!(ids.len() > 4 * SCAN_BATCH);
        backend.delete_card("book", &ids[11], None).unwrap();
        assert_streams_as_it_listed(&mut backend);
        assert_eq!(backend.list_cards("book", None, None).unwrap().len(), 39);
    }

    /// Makes the bodies of `ids` unreadable: each blob turns into a
    /// directory, so any read of it fails the listing.
    fn poison(backend: &PimdirBackend, ids: &[String]) {
        for id in ids {
            let item = backend.item("book", id).unwrap();
            let path = backend
                .inner
                .reader
                .blobs()
                .path(item.object.as_ref().unwrap());
            std::fs::remove_file(&path).unwrap();
            std::fs::create_dir(&path).unwrap();
        }
    }

    /// A page reads the bodies of its own cards and no other: not those
    /// before it, not those after it.
    #[test]
    fn a_page_reads_no_body_but_its_own() {
        let (_dir, mut backend) = book(40);
        let order: Vec<String> = backend
            .list_cards("book", None, None)
            .unwrap()
            .into_iter()
            .map(|card| card.id)
            .collect();
        let page = backend.list_cards("book", Some(3), Some(5)).unwrap();
        assert_eq!(
            page.iter().map(|card| card.id.clone()).collect::<Vec<_>>(),
            order[10..15]
        );

        poison(&backend, &order[..10]);
        poison(&backend, &order[15..]);

        assert_eq!(backend.list_cards("book", Some(3), Some(5)).unwrap(), page);
        assert!(eager(&backend, Some(3), Some(5)).is_err());
        assert!(backend.list_cards("book", Some(2), Some(5)).is_err());
    }

    /// A card in the address book `local`, which the device keeps and no
    /// source syncs (pimdir STORAGE §14.3), answering its public id.
    fn kept(dir: &tempfile::TempDir) -> String {
        let mut store = io_pimdir::client::PimdirStore::open(dir.path()).unwrap();
        store.ensure_collection("local", CARD_KIND).unwrap();
        let (_, seq) = store.put_item("local", JANE).unwrap();
        seq.to_string()
    }

    /// An address book no source syncs takes no write: the owner would
    /// apply it as a source that never lists the collection, so the card
    /// would never reach a server.
    #[test]
    fn a_write_into_an_address_book_no_source_syncs_is_refused() {
        let (dir, mut backend) = store();
        let id = kept(&dir);
        let created = card(1, "Joe Bloggs");

        let err = backend
            .create_card("local", created.clone())
            .unwrap_err()
            .to_string();
        assert!(err.contains("no source syncs"), "{err}");
        let err = backend
            .update_card("local", &id, card(2, "Jane Roe"), None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("no source syncs"), "{err}");
        let err = backend
            .delete_card("local", &id, None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("no source syncs"), "{err}");

        let producer = backend.inner.producer().unwrap();
        assert!(producer.pending_actions("local").unwrap().is_empty());
        let blob = backend.inner.reader.blobs().path(&producer.hash(&created));
        assert!(!blob.exists(), "a refused create leaves no body behind");
    }

    /// A card whose blob is gone from the store has no local body: it
    /// reports no version and matches none, as an unfetched one does.
    #[test]
    fn a_card_whose_blob_is_missing_has_no_version() {
        use pimalaya_cli::error::code_of;

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drain(&dir);
        let id = backend.list_cards("book", None, None).unwrap()[0]
            .id
            .clone();
        let hash = backend.item("book", &id).unwrap().object.unwrap();
        std::fs::remove_file(backend.inner.reader.blobs().path(&hash)).unwrap();

        let listed = backend.list_cards("book", None, None).unwrap();
        assert_eq!(listed[0].etag, None);

        let err = backend.delete_card("book", &id, Some(&hash.0)).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::PreconditionFailed));
        assert_eq!(pending(&backend), 0);
    }

    /// A kind is a media type, compared before its parameters as io-pimdir
    /// compares it when it picks the summaries; a calendar is no address
    /// book.
    #[test]
    fn an_address_book_is_told_by_its_media_type() {
        let dir = tempfile::tempdir().unwrap();
        {
            let store = io_pimdir::client::PimdirStore::open(dir.path()).unwrap();
            store.ensure_collection("versioned", CARD_KIND).unwrap();
            store
                .ensure_collection("calendar", "text/calendar")
                .unwrap();
        }

        // NOTE: a store at storage-draft-05 closes the kind to bare media
        // types, but one an earlier io-pimdir wrote is read as it is, its
        // kinds not yet repaired by its owner.
        assert!(is_card_kind("text/vcard; version=4.0"));
        assert!(!is_card_kind("text/calendar; component=VEVENT"));

        let mut backend = PimdirBackend::new(PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        })
        .unwrap();
        let ids: Vec<String> = backend
            .list_addressbooks()
            .unwrap()
            .into_iter()
            .map(|book| book.id)
            .collect();
        assert_eq!(ids, ["versioned"]);
        assert!(backend.list_cards("calendar", None, None).is_err());
    }

    /// One CLI view is one account: with nothing configured a store
    /// grouping several is refused, naming them, until `pimdir.account`
    /// picks one.
    #[test]
    fn an_unset_account_is_refused_on_a_store_grouping_several() {
        let dir = tempfile::tempdir().unwrap();
        for (account, id) in [("work", "work/book"), ("home", "home/book")] {
            io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_account(account)
                .ensure_collection(id, CARD_KIND)
                .unwrap();
        }
        let open = |account: Option<&str>| {
            PimdirBackend::new(PimdirConfig {
                root: dir.path().to_path_buf(),
                account: account.map(String::from),
            })
        };

        let err = match open(None) {
            Ok(_) => panic!("an unset account opened a store grouping two"),
            Err(err) => err.to_string(),
        };
        assert!(err.contains("home, work"), "{err}");

        let ids: Vec<String> = open(Some("work"))
            .unwrap()
            .list_addressbooks()
            .unwrap()
            .into_iter()
            .map(|book| book.id)
            .collect();
        assert_eq!(ids, ["work/book"]);
    }

    /// What a command printed, as text and as JSON.
    #[derive(Default)]
    struct Captured {
        text: String,
        json: serde_json::Value,
    }

    impl pimalaya_cli::printer::Printer for Captured {
        fn out<T: std::fmt::Display + serde::Serialize>(&mut self, data: T) -> Result<()> {
            self.text = data.to_string();
            self.json = serde_json::to_value(&data)?;
            Ok(())
        }
    }

    /// A shared client over the store at `dir`, as a command gets one.
    fn client(dir: &tempfile::TempDir) -> crate::shared::client::AddressbookClient {
        crate::shared::client::AddressbookClient::new(
            crate::config::Config::default(),
            crate::config::AccountConfig {
                pimdir: Some(PimdirConfig {
                    root: dir.path().to_path_buf(),
                    account: None,
                }),
                ..Default::default()
            },
            crate::backend::Backend::Auto,
        )
        .unwrap()
    }

    /// A pimdir write reaches the server on the next sync, so the shared
    /// commands say it is queued, never that the card was written, while
    /// their JSON stays the shape every backend prints.
    #[test]
    fn a_queued_write_says_it_is_queued() {
        use clap::Parser;

        use crate::shared::card::{
            create::CardCreateCommand, delete::CardDeleteCommand, update::CardUpdateCommand,
        };

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drain(&dir);
        let id = backend.list_cards("book", None, None).unwrap()[0]
            .id
            .clone();
        let joe = String::from_utf8(card(1, "Joe Bloggs")).unwrap();
        let roe = String::from_utf8(JANE.to_vec())
            .unwrap()
            .replace("Jane Doe", "Jane Roe");
        let mut out = Captured::default();

        CardCreateCommand::parse_from(["create", "-k", "book", &joe])
            .execute(&mut out, client(&dir))
            .unwrap();
        assert!(out.text.contains("queued"), "{}", out.text);
        assert!(!out.text.contains("successfully"), "{}", out.text);
        assert_eq!(out.json, serde_json::json!({ "id": "urn:uuid:card-1" }));

        CardUpdateCommand::parse_from(["update", "-k", "book", &id, &roe])
            .execute(&mut out, client(&dir))
            .unwrap();
        assert!(out.text.contains("queued"), "{}", out.text);
        assert!(!out.text.contains("successfully"), "{}", out.text);
        assert_eq!(
            out.json,
            serde_json::json!({ "id": id, "keptProperties": [] })
        );

        CardDeleteCommand::parse_from(["delete", "-k", "book", &id])
            .execute(&mut out, client(&dir))
            .unwrap();
        assert!(out.text.contains("queued"), "{}", out.text);
        assert!(!out.text.contains("successfully"), "{}", out.text);
    }

    /// A create naming a `UID` the address book already holds, queued or
    /// applied, is refused with its code and queues nothing: the owner
    /// would park it at drain (pimdir STORAGE §15.3), so accepting it
    /// would report a card no sync ever creates.
    #[test]
    fn a_create_naming_a_held_uid_is_refused() {
        use pimalaya_cli::error::code_of;

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();

        let err = backend.create_card("book", JANE.to_vec()).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::ItemDuplicate), "{err:#}");
        assert_eq!(pending(&backend), 1);

        drain(&dir);
        let roe = String::from_utf8(JANE.to_vec())
            .unwrap()
            .replace("Jane Doe", "Jane Roe");
        let before = objects(&dir);
        let err = backend.create_card("book", roe.into_bytes()).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::ItemDuplicate), "{err:#}");
        assert_eq!(pending(&backend), 0);
        assert_eq!(objects(&dir), before, "a refused create leaves no body");
        assert_eq!(backend.list_cards("book", None, None).unwrap().len(), 1);
    }

    /// Every file of the store's blob tree, sorted.
    fn objects(dir: &tempfile::TempDir) -> Vec<std::path::PathBuf> {
        fn walk(path: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    out.push(path);
                }
            }
        }

        let mut out = Vec::new();
        walk(&dir.path().join("objects"), &mut out);
        out.sort();
        out
    }

    /// Writes `body` as the card `jane.vcf` of `collection`, as a sync
    /// pulling it from the server leaves it: stored, hydrated and clean.
    fn pulled(dir: &tempfile::TempDir, collection: &str, body: &[u8]) {
        use io_pimdir::{
            change::PimdirWriteOp,
            collection::PimdirCollectionId,
            object::PimdirObject,
            placement::{PimdirBase, PimdirHandle, PimdirPlacement, PimdirStatus},
        };

        let mut store = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("carddav");
        let hash = store.hash(body);
        let derived = contact::derive(body);
        store
            .write(vec![
                PimdirWriteOp::StoreObject {
                    object: PimdirObject {
                        hash: hash.clone(),
                        size: body.len(),
                    },
                    body: Some(body.to_vec()),
                },
                PimdirWriteOp::UpsertPlacement(PimdirPlacement {
                    collection: PimdirCollectionId::from(collection),
                    handle: PimdirHandle("jane.vcf".into()),
                    link_id: Some(derived.link_id),
                    object: Some(hash.clone()),
                    level: PimdirLevel::Full,
                    summary: derived.summary,
                    sort_key: Default::default(),
                    flags: PimdirFlags::default(),
                    status: PimdirStatus::Clean,
                    conflict_revision: None,
                    conflict_object: None,
                    base: Some(PimdirBase {
                        flags: PimdirFlags::default(),
                        revision: None,
                        object: Some(hash),
                    }),
                    origin: None,
                }),
            ])
            .unwrap();
    }

    /// A queued edit names the body it was made against (pimdir STORAGE
    /// §15.3): a server change a sync brings in before the drain parks the
    /// edit, rather than being overwritten without a merge.
    #[test]
    fn an_edit_whose_body_a_sync_replaced_is_parked() {
        let (dir, mut backend) = store();
        pulled(&dir, "book", JANE);
        let id = backend.list_cards("book", None, None).unwrap()[0]
            .id
            .clone();
        let read = backend.get_card("book", &id).unwrap();
        let rename = |name: &str| {
            String::from_utf8(JANE.to_vec())
                .unwrap()
                .replace("Jane Doe", name)
                .into_bytes()
        };

        backend
            .update_card("book", &id, rename("Jane Roe"), read.etag.as_deref())
            .unwrap();
        pulled(&dir, "book", &rename("Jane Poe"));

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("carddav");
        let report = owner.drain().unwrap();
        assert_eq!((report.applied, report.parked), (0, 1), "{report:?}");
        assert_eq!(
            backend.get_card("book", &id).unwrap().contents,
            rename("Jane Poe")
        );
    }

    /// A guarded edit the owner parks, its body replaced by a sync before
    /// the drain, shows in `pimdir status` with its row and the reason it
    /// never applied, and only in the status of the account it targets.
    #[test]
    fn status_shows_the_parked_edit_of_the_account_read() {
        let dir = tempfile::tempdir().unwrap();
        let open = |account: &str| {
            PimdirBackend::new(PimdirConfig {
                root: dir.path().to_path_buf(),
                account: Some(account.into()),
            })
            .unwrap()
        };
        let rename = |name: &str| {
            String::from_utf8(JANE.to_vec())
                .unwrap()
                .replace("Jane Doe", name)
                .into_bytes()
        };

        for (account, book) in [("work", "work/book"), ("home", "home/book")] {
            synced(dir.path(), Some(account), book);
            pulled(&dir, book, JANE);

            let mut backend = open(account);
            let id = backend.list_cards(book, None, None).unwrap()[0].id.clone();
            let read = backend.get_card(book, &id).unwrap();
            backend
                .update_card(book, &id, rename("Jane Roe"), read.etag.as_deref())
                .unwrap();
            pulled(&dir, book, &rename("Jane Poe"));
        }
        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("carddav");
        assert_eq!(owner.drain().unwrap().parked, 2);
        drop(owner);

        let status = open("work").status().unwrap();
        assert_eq!(status.account.as_deref(), Some("work"));
        assert_eq!(status.accounts, ["home", "work"]);
        let books: Vec<_> = status
            .addressbooks
            .iter()
            .map(|book| (book.id.as_str(), book.total, book.hydrated, book.queued))
            .collect();
        assert_eq!(books, [("work/book", 1, 1, 0)]);

        assert_eq!(status.parked.len(), 1, "{:?}", status.parked);
        let parked = &status.parked[0];
        assert_eq!(parked.addressbook_id, "work/book");
        assert_eq!(parked.action, "update");
        assert!(
            parked.error.contains("body replaced since it was read"),
            "{}",
            parked.error
        );

        let text = status.to_string();
        assert!(
            text.contains(&format!("ROW {} update in work/book", parked.row)),
            "{text}"
        );
        assert!(text.contains(&parked.error), "{text}");

        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["parked"][0]["addressbookId"], "work/book");
        assert_eq!(json["parked"][0]["row"], parked.row);
    }

    /// A composed card refused as a duplicate keeps its composer file,
    /// named in the error, as any failed write does: the edit is not lost
    /// to a success that never was.
    #[cfg(unix)]
    #[test]
    fn a_composed_duplicate_keeps_its_file() {
        use clap::Parser;
        use pimalaya_cli::error::code_of;

        use crate::shared::card::create::CardCreateCommand;

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        let jane = String::from_utf8(JANE.to_vec()).unwrap();
        let mut out = Captured::default();

        let err = CardCreateCommand::parse_from([
            "create",
            "-k",
            "book",
            "-i",
            "--composer",
            // NOTE: a suffix glued to `-i` is what GNU and BSD sed both
            // read as in-place.
            "sed -i.bak s/Doe/Roe/",
            &jane,
        ])
        .execute(&mut out, client(&dir))
        .unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::ItemDuplicate), "{err:#}");

        let message = err.to_string();
        let path = message
            .split('"')
            .nth(1)
            .unwrap_or_else(|| panic!("no composer file named in: {message}"));
        let kept = std::fs::read_to_string(path).unwrap();
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(format!("{path}.bak")).unwrap();
        assert!(kept.contains("FN:Jane Roe"), "{kept}");
        assert_eq!(pending(&backend), 1);
    }

    /// A root holding no store, a database declaring tables at
    /// `user_version` 0 (another program's) and a write a syncing source
    /// does not support (pimdir STORAGE §15.6) each carry the code
    /// io-pimdir names, and none asks to delete anything.
    #[test]
    fn a_store_refusal_carries_its_code() {
        use io_pimdir::capability::{self, PimdirCapability, PimdirSupport};
        use pimalaya_cli::error::code_of;

        let empty = tempfile::tempdir().unwrap();
        let err = match PimdirBackend::new(PimdirConfig {
            root: empty.path().to_path_buf(),
            account: None,
        }) {
            Ok(_) => panic!("a root holding no store opened"),
            Err(err) => err,
        };
        assert_eq!(code_of(&err), Some(ErrorCode::StoreUncreated), "{err:#}");
        assert!(!format!("{err:#}").contains("delete"), "{err:#}");

        // NOTE: SQLite keeps `PRAGMA user_version` big-endian at offset 60
        // of the database header.
        let (foreign, _) = store();
        let mut db = std::fs::OpenOptions::new()
            .write(true)
            .open(foreign.path().join("pimdir.db"))
            .unwrap();
        std::io::Seek::seek(&mut db, std::io::SeekFrom::Start(60)).unwrap();
        std::io::Write::write_all(&mut db, &0u32.to_be_bytes()).unwrap();
        drop(db);
        let err = match PimdirBackend::new(PimdirConfig {
            root: foreign.path().to_path_buf(),
            account: None,
        }) {
            Ok(_) => panic!("a foreign database opened"),
            Err(err) => err,
        };
        assert_eq!(code_of(&err), Some(ErrorCode::StoreForeign), "{err:#}");
        assert!(!format!("{err:#}").contains("delete"), "{err:#}");

        let (dir, mut backend) = store();
        let declaration: Vec<PimdirCapability> = capability::CONTACTS
            .iter()
            .map(|name| PimdirCapability {
                collection: None,
                name: name.to_string(),
                support: match *name == capability::CONTACTS_CARD_ADD {
                    true => PimdirSupport::None,
                    false => PimdirSupport::Full,
                },
                detail: None,
            })
            .collect();
        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .declare("carddav", &declaration)
            .unwrap();

        let err = backend.create_card("book", JANE.to_vec()).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::Unsupported), "{err:#}");
        assert!(format!("{err:#}").contains(capability::CONTACTS_CARD_ADD));
        assert_eq!(pending(&backend), 0);
    }
}
