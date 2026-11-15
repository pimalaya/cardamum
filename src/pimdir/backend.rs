//! # Pimdir backend
//!
//! Pimdir arm of the unified-API client, mapping the unified addressbook
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

use anyhow::{Error, Result, anyhow, bail};
use io_pimdir::{
    client::{
        PimdirError, PimdirStore,
        producer::PimdirActionStatus,
        reader::{PimdirCollection, PimdirItem},
    },
    codec::{PimdirAction, action_from_payload},
    object::PimdirHash,
    placement::PimdirFlags,
    summary::{PimdirSummary, contact},
};
use log::warn;
use pimalaya_cli::error::CodedError;

use crate::{
    config::{AccountConfig, PimdirConfig},
    error::{self, ErrorCode},
    pimdir::{
        client::{PimdirClient, refusal},
        queue::list::PimdirQueueRow,
        status::{
            PimdirAddressbookStatus, PimdirParkedStatus, PimdirRefusedStatus, PimdirStatusOutput,
        },
    },
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
        client::CardWindow,
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

/// Pimdir backend of the unified-API client, over an opened local store.
pub struct PimdirBackend {
    pub(crate) inner: PimdirClient,
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
    pub fn build(name: String, account_config: &mut AccountConfig) -> Result<Self> {
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
    /// books, how much of each is downloaded and queued, the actions the
    /// store's owner parked in them, and the refusals standing there.
    ///
    /// A parked action is one the owner will never apply (pimdir STORAGE
    /// §15.3), so the write a command reported as queued is lost unless
    /// something shows it. Narrowed to the account's address books, as
    /// every other read is.
    pub fn status(&mut self) -> Result<PimdirStatusOutput> {
        let accounts = self.inner.reader.list_accounts()?;
        let mut addressbooks = Vec::new();
        let mut refused = Vec::new();

        for collection in self.collections()? {
            let refusals = self
                .inner
                .reader
                .refusals(&collection.id)
                .map_err(|err| refusal(err, "Read the pimdir refusals"))?;
            refused.extend(refusals.into_iter().map(|row| PimdirRefusedStatus {
                addressbook_id: collection.id.clone(),
                item_id: row.seq.to_string(),
                source: row.source,
                reason: row.reason,
                refused_at: row.refused_at,
            }));

            let (mut total, mut hydrated) = (0, 0);
            self.scan(&collection.id, |item| {
                total += 1;
                hydrated += usize::from(item.object.is_some());
                Ok(ControlFlow::Continue(()))
            })?;
            let queued = self
                .inner
                .reader
                .count_pending_creates(&collection.id)
                .map_err(|err| refusal(err, "Read the pimdir queue"))?;

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
                kind: parked.action,
                error: parked.error,
            })
            .collect();

        Ok(PimdirStatusOutput {
            account: self.inner.account.clone(),
            accounts,
            addressbooks,
            parked,
            refused,
        })
    }

    /// The ids of the account's address books a queue command reads,
    /// `addressbook` alone when named, refused when the account holds no
    /// such address book.
    fn queue_collections(&self, addressbook: Option<&str>) -> Result<Vec<String>> {
        if let Some(addressbook) = addressbook {
            self.known_collection(addressbook)?;
            return Ok(vec![addressbook.to_owned()]);
        }

        let mut ids: Vec<String> = self.collections()?.into_iter().map(|c| c.id).collect();
        ids.sort();
        Ok(ids)
    }

    /// The card creations staged in the account's address books, or in
    /// `addressbook` alone, in the order they were queued.
    ///
    /// A queued creation has no public id until the owner applies it, so
    /// no listing shows it: this is where it does, by its queue row, the
    /// link id `card add` reported, and the full name of the body it pins
    /// when that body is local.
    pub fn queued_cards(&self, addressbook: Option<&str>) -> Result<Vec<PimdirQueueRow>> {
        let mut cards = Vec::new();

        for collection in self.queue_collections(addressbook)? {
            let pending = self
                .inner
                .reader
                .pending_creates(&collection)
                .map_err(|err| refusal(err, "Read the pimdir queue"))?;

            for queued in pending {
                let kind = queued.action.kind().to_owned();
                let target = self.row_target(&queued.collection, queued.action)?;

                cards.push(PimdirQueueRow {
                    id: queued.id,
                    kind,
                    collection: queued.collection,
                    link_id: target.link_id,
                    item_id: target.item_id,
                    producer: queued.producer,
                    queued_at: queued.created_at,
                    attempts: u64::try_from(queued.attempts).unwrap_or_default(),
                    unconfirmed: false,
                    error: None,
                    full_name: target.full_name,
                });
            }
        }

        cards.sort_by_key(|card| card.id);
        Ok(cards)
    }

    /// The rows of any kind the owner parked in the account's address
    /// books, or in `addressbook` alone, in the order they were queued.
    ///
    /// The row's payload is read for the card it creates or addresses: one
    /// that does not decode, or an intent, names none.
    pub fn parked_rows(&self, addressbook: Option<&str>) -> Result<Vec<PimdirQueueRow>> {
        let collections = self.queue_collections(addressbook)?;
        let mut rows = Vec::new();

        for parked in self.inner.reader.parked_actions()? {
            if !collections.contains(&parked.collection) {
                continue;
            }

            let target = match action_from_payload(&parked.action, &parked.payload) {
                Ok(action) => self.row_target(&parked.collection, action)?,
                Err(_) => PimdirRowTarget::default(),
            };

            rows.push(PimdirQueueRow {
                unconfirmed: parked.unconfirmed().is_some(),
                id: parked.id,
                kind: parked.action,
                collection: parked.collection,
                link_id: target.link_id,
                item_id: target.item_id,
                producer: parked.producer,
                queued_at: parked.created_at,
                attempts: u64::try_from(parked.attempts).unwrap_or_default(),
                error: Some(parked.error),
                full_name: target.full_name,
            });
        }

        Ok(rows)
    }

    /// The card a queued `action` on `collection` creates or addresses,
    /// and its full name: read from the body the action pins, else from
    /// the summary of the card it addresses.
    fn row_target(&self, collection: &str, action: PimdirAction) -> Result<PimdirRowTarget> {
        let (link_id, seq, object) = match action {
            PimdirAction::Add {
                link_id, object, ..
            } => (link_id.map(|link_id| link_id.0), None, object),
            PimdirAction::Update { seq, object, .. } => (None, Some(seq), Some(object)),
            PimdirAction::SetFlags { seq, .. }
            | PimdirAction::Remove { seq, .. }
            | PimdirAction::Move { seq, .. }
            | PimdirAction::Copy { seq, .. } => (None, Some(seq), None),
            PimdirAction::SetPerformer { .. } | PimdirAction::Unknown { .. } => {
                return Ok(PimdirRowTarget::default());
            }
        };

        let full_name = match (object, seq) {
            (Some(hash), _) => {
                let body = self.inner.reader.blobs().get(&hash)?;
                body.and_then(|body| contact_full_name(contact::derive(&body).summary))
            }
            (None, Some(seq)) => self
                .inner
                .reader
                .get_item(collection, seq)?
                .and_then(|item| contact_full_name(item.summary)),
            (None, None) => None,
        };

        Ok(PimdirRowTarget {
            link_id,
            item_id: seq.map(|seq| seq.to_string()),
            full_name,
        })
    }

    /// Where the queue row `id` stands: pending, parked, unconfirmed,
    /// applied with the card it acted on and the refusals standing there,
    /// or unknown.
    ///
    /// A row anchored on a collection that is not one of the account's
    /// address books reads as unknown, as another account's row reads in
    /// Himalaya. A row applied on a collection gone since names none, so
    /// no account can be told from it, and it reads as applied.
    pub fn queue_row(&self, id: i64) -> Result<PimdirActionStatus> {
        let status = self
            .inner
            .reader
            .action_status(id)
            .map_err(|err| refusal(err, format!("Read queue row {id}")))?;

        let collection = match &status {
            PimdirActionStatus::Pending { collection, .. }
            | PimdirActionStatus::Parked { collection, .. }
            | PimdirActionStatus::Unconfirmed { collection, .. }
            | PimdirActionStatus::Applied {
                collection: Some(collection),
                ..
            } => collection,
            PimdirActionStatus::Applied {
                collection: None, ..
            }
            | PimdirActionStatus::Unknown
            | PimdirActionStatus::Reset => return Ok(status),
        };

        let ours = self
            .collections()?
            .iter()
            .any(|ours| &ours.id == collection);

        Ok(match ours {
            true => status,
            false => PimdirActionStatus::Unknown,
        })
    }

    /// Cancels one pending or parked row of the account's queue, returning
    /// its action kind.
    ///
    /// The row is read first, so one of another account, or one applied
    /// already, is refused before the owner role is taken. The store is
    /// opened as it stands, never created nor migrated, and a sync in
    /// flight owning it is refused at once rather than waited out: the
    /// action may have been applied by the time the message is read.
    pub fn cancel_queued(&self, id: i64) -> Result<String> {
        let kind = match self.queue_row(id)? {
            PimdirActionStatus::Pending { kind, .. }
            | PimdirActionStatus::Parked { kind, .. }
            | PimdirActionStatus::Unconfirmed { kind, .. } => kind,
            PimdirActionStatus::Applied { .. } => bail!("Row {id} was applied already"),
            // NOTE: a bare row id states no store, so io-pimdir follows it
            // unchecked and never answers a reset for it.
            PimdirActionStatus::Unknown | PimdirActionStatus::Reset => bail!(
                "No queued action with row {id} in this account's queue; \
                 it may have been synced already"
            ),
        };

        let root = &self.inner.root;
        let context = format!("Cancel queued action {id}");

        let mut owner = PimdirStore::open_current(root).map_err(|err| match err {
            PimdirError::Owned(_) => {
                let message = format!(
                    "A sync is running on `{}`, so the queue cannot be edited; \
                     the action may have been applied already",
                    root.display(),
                );
                CodedError::new(ErrorCode::StoreOwned, message).into()
            }
            err => refusal(err, context.clone()),
        })?;

        let dropped = owner
            .drop_action(id)
            .map_err(|err| refusal(err, context.clone()))?;

        if !dropped {
            bail!("No queued action with row {id}; it may have been synced already");
        }

        Ok(kind)
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

    /// Lists the `take` cards inside `addressbook_id` after the first `skip`.
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
        mut skip: usize,
        take: Option<usize>,
    ) -> Result<CardWindow> {
        self.known_collection(addressbook_id)?;

        if take == Some(0) {
            return Ok(CardWindow::default());
        }

        let mut cards = Vec::new();
        let mut errors = Vec::new();

        self.scan(addressbook_id, |item| {
            if skip > 0 {
                skip -= 1;
                return Ok(ControlFlow::Continue(()));
            }

            let (card, err) = self.listed_card(addressbook_id, item);

            if let Some(err) = err {
                errors.push(error::unreadable_card(Some(&card.id), err));
            }

            cards.push(card);

            match take {
                Some(take) if cards.len() >= take => Ok(ControlFlow::Break(())),
                _ => Ok(ControlFlow::Continue(())),
            }
        })?;

        Ok(CardWindow { cards, errors })
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
            resolves: None,
        })?;

        Ok(CardUpdateOutcome::default())
    }

    /// Stages a `remove` action for `card_id`.
    ///
    /// The next sync applies it as a tombstone and pushes a server-side
    /// delete. `if_match` gates it as it gates
    /// [`update_card`](Self::update_card), and the action names the body
    /// it matched the same way (pimdir STORAGE §15.3), so the owner parks
    /// a delete decided over a body a sync replaced before the drain.
    ///
    /// It states no resolution: over a conflicted card the owner parks
    /// it, as it parks an update, rather than delete a diverging body
    /// nobody saw. The partial supports the gate passed it with are
    /// logged.
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
        // NOTE: the version matched, so the item's body is the one read.
        let read = if_match.and(item.object);
        let action = PimdirAction::Remove {
            seq,
            if_match: read,
            resolves: None,
        };
        let enqueued = self
            .inner
            .producer()?
            .enqueue(addressbook_id, &action, None)
            .map_err(|err| refusal(err, "Stage the pimdir action"))?;

        for partial in &enqueued.partial {
            warn!("{partial}");
        }
        Ok(())
    }

    /// The address book collections of the configured account.
    ///
    /// One store holds every kind a sync caches (pimdir STORAGE §4.3, §14),
    /// so the kind separates an address book from a mailbox or a calendar.
    /// A kind-less one counts: a sync predating kinds left the column empty.
    pub(crate) fn collections(&self) -> Result<Vec<PimdirCollection>> {
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

    /// Builds a shared [`Card`] from a stored item as a listing shows it,
    /// beside the error its body could not be read with: the real body when
    /// it is local, else a preview projected from the stored summary, which
    /// keeps a listing useful on a partly synced store. The preview is never
    /// the record: [`get_card`](Self::get_card) refuses an unhydrated card
    /// outright.
    ///
    /// A body that cannot be read, or that its blob tree no longer holds
    /// though the item names one, gives the preview an unhydrated card
    /// lists as, the error left to the caller: a listing keeps the card's
    /// place in the page and prints it in `errors` alone, as calendula
    /// names such an item. A body not fetched yet is no error.
    pub(crate) fn listed_card(
        &self,
        addressbook_id: &str,
        item: PimdirItem,
    ) -> (Card, Option<Error>) {
        let (stored, err) = match &item.object {
            Some(hash) => match self.inner.reader.blobs().get(hash) {
                Ok(Some(stored)) => (Some(stored), None),
                Ok(None) => (None, Some(Error::new(BodyMissing))),
                Err(err) => {
                    let err = Error::from(err).context("Cannot read the card body");
                    (None, Some(err))
                }
            },
            None => (None, None),
        };
        // NOTE: the version is the body's, so a card whose blob is gone
        // has none, as an unfetched one has none.
        let etag = match stored {
            Some(_) => item.object.as_ref().map(|hash| hash.0.clone()),
            None => None,
        };

        let contents = match stored {
            Some(contents) => contents,
            None => preview_vcard(&item),
        };

        let card = Card {
            id: item.seq.to_string(),
            addressbook_id: addressbook_id.to_string(),
            etag,
            contents,
        };

        (card, err)
    }

    /// Fails unless the store knows `collection`, naming those it holds.
    ///
    /// The read seam answers an unknown collection with an empty page and
    /// the queue accepts any name, so without this a typo in `-k` would read
    /// as an empty addressbook and stage what nothing will ever apply. The
    /// ids carry the sync engine's namespace, so the refusal lists them.
    pub(crate) fn known_collection(&self, collection: &str) -> Result<()> {
        let mut ids: Vec<String> = self
            .collections()?
            .into_iter()
            .map(|candidate| candidate.id)
            .collect();

        if ids.iter().any(|id| id == collection) {
            return Ok(());
        }

        ids.sort();

        let message = format!(
            "Addressbook `{collection}` not found; this account holds: {}",
            ids.join(", "),
        );
        Err(CodedError::new(ErrorCode::AddressbookNotFound, message).into())
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
        let missing =
            |message: String| Error::new(CodedError::new(ErrorCode::CardNotFound, message));

        // NOTE: a card id is the store's number for it, so one that is no
        // number names no card.
        let seq = card_id
            .parse::<i64>()
            .map_err(|_| missing(format!("Invalid card id `{card_id}` (expected a number)")))?;

        self.inner.reader.get_item(collection, seq)?.ok_or_else(|| {
            missing(format!(
                "Card `{card_id}` not found in address book `{collection}`"
            ))
        })
    }

    /// Stages a body and the action naming it as one step.
    ///
    /// `PimdirProducer::enqueue_with_body` checks the action first: the
    /// gate of pimdir STORAGE §15.6, run once, and, for an `add`, a link id
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
            .enqueue_with_body(collection, contents, action)
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

/// The card a queue row creates or addresses, as `pimdir queue list`
/// prints it.
#[derive(Debug, Default)]
struct PimdirRowTarget {
    /// The link id of the card the row creates.
    link_id: Option<String>,
    /// The public id of the stored card the row addresses.
    item_id: Option<String>,
    /// The card's full name.
    full_name: Option<String>,
}

/// The full name a summary holds, when it is a contact's.
fn contact_full_name(summary: Option<PimdirSummary>) -> Option<String> {
    match summary? {
        PimdirSummary::Contact(summary) => Some(summary.full_name),
        _ => None,
    }
}

/// A body an item names that its blob tree no longer holds, an
/// inconsistent store.
#[derive(Debug)]
struct BodyMissing;

impl std::fmt::Display for BodyMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Body blob missing from the store")
    }
}

impl std::error::Error for BodyMissing {}

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
    use io_pimdir::{
        placement::{PimdirLevel, PimdirLinkId},
        sql::FORMAT_VERSION,
    };

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

        let cards = backend.list_cards("book", 0, None).unwrap().cards;
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

        let listed = backend.list_cards("book", 0, None).unwrap().cards;
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
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
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
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
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
        assert!(
            backend
                .list_cards("book", 0, None)
                .unwrap()
                .cards
                .is_empty()
        );

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
    fn eager(backend: &PimdirBackend, skip: usize, take: Option<usize>) -> Result<Vec<Card>> {
        let mut all = Vec::new();
        backend.scan("book", |item| {
            all.push(item);
            Ok(ControlFlow::Continue(()))
        })?;

        let cards = all
            .into_iter()
            .map(|item| match backend.listed_card("book", item) {
                (card, None) => Ok(card),
                (_, Some(err)) => Err(err),
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(crate::shared::client::window(cards, skip, take))
    }

    /// Every page comes out as it did when the whole address book was
    /// read first: same cards, same order, same bodies and versions.
    fn assert_streams_as_it_listed(backend: &mut PimdirBackend) {
        let mut listed = 0;
        for take in [
            None,
            Some(0),
            Some(1),
            Some(3),
            Some(7),
            Some(10),
            Some(100),
        ] {
            for skip in [0, 1, 2, 5, 9, 50, 400] {
                let streamed = backend.list_cards("book", skip, take).unwrap().cards;
                let expected = eager(backend, skip, take).unwrap();
                assert_eq!(streamed, expected, "{take:?} after {skip}");
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
            .list_cards("book", 0, None)
            .unwrap()
            .cards
            .into_iter()
            .map(|card| card.id)
            .collect();
        backend
            .update_card("book", &ids[3], card(3, "Renamed"), None)
            .unwrap();
        assert!(ids.len() > 4 * SCAN_BATCH);
        backend.delete_card("book", &ids[11], None).unwrap();
        assert_streams_as_it_listed(&mut backend);
        assert_eq!(backend.list_cards("book", 0, None).unwrap().cards.len(), 39);
    }

    /// Makes the bodies of `ids` unreadable: each blob loses every
    /// permission, so any read of it fails.
    ///
    /// A directory where a blob belongs no longer does, io-pimdir reading
    /// it as a stray, so as an absent body (pimdir STORAGE §7).
    #[cfg(unix)]
    fn poison(backend: &PimdirBackend, ids: &[String]) {
        use std::os::unix::fs::PermissionsExt;

        for id in ids {
            let item = backend.item("book", id).unwrap();
            let path = backend
                .inner
                .reader
                .blobs()
                .path(item.object.as_ref().unwrap())
                .unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
        }
    }

    /// A page reads the bodies of its own cards and no other: not those
    /// before it, not those after it.
    #[cfg(unix)]
    #[test]
    fn a_page_reads_no_body_but_its_own() {
        let (_dir, mut backend) = book(40);
        let order: Vec<String> = backend
            .list_cards("book", 0, None)
            .unwrap()
            .cards
            .into_iter()
            .map(|card| card.id)
            .collect();
        let page = backend.list_cards("book", 10, Some(5)).unwrap().cards;
        assert_eq!(
            page.iter().map(|card| card.id.clone()).collect::<Vec<_>>(),
            order[10..15]
        );

        poison(&backend, &order[..10]);
        poison(&backend, &order[15..]);

        let listed = backend.list_cards("book", 10, Some(5)).unwrap();
        assert_eq!(listed.cards, page);
        assert!(listed.errors.is_empty(), "{:?}", listed.errors);
        assert!(eager(&backend, 10, Some(5)).is_err());
        assert_eq!(
            backend.list_cards("book", 5, Some(5)).unwrap().errors.len(),
            5
        );
    }

    /// A body that cannot be read is an entry on the page holding its
    /// card. The window keeps the card's place with its preview and no
    /// version, and the page prints it in `errors` alone.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_body_is_an_entry_alone() {
        use crate::error::ErrorCode;

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drain(&dir);
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        poison(&backend, std::slice::from_ref(&id));

        let listed = backend.list_cards("book", 0, None).unwrap();
        assert_eq!(listed.cards.len(), 1);
        assert_eq!(listed.cards[0].id, id);
        assert_eq!(listed.cards[0].etag, None);
        let preview = String::from_utf8(listed.cards[0].contents.clone()).unwrap();
        assert!(preview.contains("FN:Jane Doe\r\n"), "{preview}");

        assert_eq!(listed.errors.len(), 1, "{:?}", listed.errors);
        let entry = &listed.errors[0];
        assert_eq!(entry.id.as_deref(), Some(id.as_str()));
        assert_eq!(entry.code, Some(ErrorCode::CardUnreadable));
        assert_eq!(entry.error, "Cannot read the card body");
        assert!(!entry.sources.is_empty());

        let page = client(&dir).list_cards("book", Some(1), Some(25)).unwrap();
        assert!(page.cards.is_empty(), "{:?}", page.cards);
        assert_eq!(page.errors.len(), 1, "{:?}", page.errors);
        assert_eq!(page.errors[0].id.as_deref(), Some(id.as_str()));
        assert_eq!(page.errors[0].code, Some(ErrorCode::CardUnreadable));
    }

    /// A body the item names but its blob tree no longer holds is an entry
    /// on the page holding its card, coded `card-unreadable` as calendula
    /// codes it `item-unreadable`, its error not repeating the id, the card
    /// keeping its preview for the paging.
    #[test]
    fn a_missing_body_is_an_entry_alone() {
        use crate::error::ErrorCode;

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drain(&dir);
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        let item = backend.item("book", &id).unwrap();
        let blob = backend
            .inner
            .reader
            .blobs()
            .path(item.object.as_ref().unwrap());
        std::fs::remove_file(blob.unwrap()).unwrap();

        let listed = backend.list_cards("book", 0, None).unwrap();
        assert_eq!(listed.cards.len(), 1);
        assert_eq!(listed.cards[0].etag, None);
        assert_eq!(listed.errors.len(), 1, "{:?}", listed.errors);
        let entry = &listed.errors[0];
        assert_eq!(entry.id.as_deref(), Some(id.as_str()));
        assert_eq!(entry.code, Some(ErrorCode::CardUnreadable));
        assert_eq!(entry.error, "Body blob missing from the store");

        let page = client(&dir).list_cards("book", Some(1), Some(25)).unwrap();
        assert!(page.cards.is_empty(), "{:?}", page.cards);
        assert_eq!(page.errors.len(), 1, "{:?}", page.errors);

        let item = backend.item("book", &id).unwrap();
        let (card, err) = backend.listed_card("book", item);
        assert!(err.is_some_and(|err| err.is::<BodyMissing>()));
        let preview = String::from_utf8(card.contents).unwrap();
        assert!(preview.contains("FN:Jane Doe\r\n"), "{preview}");
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
        let blob = backend
            .inner
            .reader
            .blobs()
            .path(&producer.hash(&created))
            .unwrap();
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
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        let hash = backend.item("book", &id).unwrap().object.unwrap();
        std::fs::remove_file(backend.inner.reader.blobs().path(&hash).unwrap()).unwrap();

        // NOTE: the body it names is gone, which the listing names as an
        // unreadable card (see a_missing_body_is_an_entry_alone).
        let listed = backend.list_cards("book", 0, None).unwrap();
        assert_eq!(listed.cards[0].etag, None);
        assert_eq!(listed.errors[0].code, Some(ErrorCode::CardUnreadable));

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
        assert!(backend.list_cards("calendar", 0, None).is_err());
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

        fn event<T: std::fmt::Display + serde::Serialize>(&mut self, _event: T) -> Result<()> {
            Ok(())
        }

        fn out_bytes(&mut self, _bytes: pimalaya_cli::output::Bytes) -> Result<()> {
            Ok(())
        }
    }

    /// A shared client over the store at `dir`, as a command gets one.
    fn client(dir: &tempfile::TempDir) -> crate::shared::client::AddressbookClient {
        crate::shared::client::AddressbookClient::new(
            crate::config::Config::default(),
            String::from("work"),
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

    /// A pimdir write reaches the server on the next sync, so the unified
    /// commands say it is queued, never that the card was written, and
    /// their JSON says so in `queued`, as every backend prints it.
    #[test]
    fn a_queued_write_says_it_is_queued() {
        use clap::Parser;

        use crate::shared::card::{
            add::CardAddCommand, delete::CardDeleteCommand, update::CardUpdateCommand,
        };

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drain(&dir);
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        let joe = String::from_utf8(card(1, "Joe Bloggs")).unwrap();
        let roe = String::from_utf8(JANE.to_vec())
            .unwrap()
            .replace("Jane Doe", "Jane Roe");
        let mut out = Captured::default();

        CardAddCommand::parse_from(["add", "-k", "book", &joe])
            .execute(&mut out, &mut client(&dir))
            .unwrap();
        assert!(out.text.contains("queued"), "{}", out.text);
        assert!(!out.text.contains("successfully"), "{}", out.text);
        assert_eq!(
            out.json,
            serde_json::json!({ "id": "urn:uuid:card-1", "queued": true })
        );

        CardUpdateCommand::parse_from(["update", "-k", "book", &id, &roe])
            .execute(&mut out, &mut client(&dir))
            .unwrap();
        assert!(out.text.contains("queued"), "{}", out.text);
        assert!(!out.text.contains("successfully"), "{}", out.text);
        assert_eq!(
            out.json,
            serde_json::json!({ "id": id, "keptProperties": [], "queued": true })
        );

        CardDeleteCommand::parse_from(["delete", "-k", "book", &id])
            .execute(&mut out, &mut client(&dir))
            .unwrap();
        assert!(out.text.contains("queued"), "{}", out.text);
        assert!(!out.text.contains("successfully"), "{}", out.text);
        assert_eq!(out.json, serde_json::json!({ "id": id, "queued": true }));
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
        assert_eq!(backend.list_cards("book", 0, None).unwrap().cards.len(), 1);
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
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
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

    /// A queued delete names the body it was decided over (pimdir STORAGE
    /// §15.3): a server change a sync brings in before the drain parks the
    /// delete, rather than removing a body nobody read.
    #[test]
    fn a_delete_whose_body_a_sync_replaced_is_parked() {
        let (dir, mut backend) = store();
        pulled(&dir, "book", JANE);
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        let read = backend.get_card("book", &id).unwrap();
        let renamed = String::from_utf8(JANE.to_vec())
            .unwrap()
            .replace("Jane Doe", "Jane Poe")
            .into_bytes();

        backend
            .delete_card("book", &id, read.etag.as_deref())
            .unwrap();
        pulled(&dir, "book", &renamed);

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("carddav");
        let report = owner.drain().unwrap();
        assert_eq!((report.applied, report.parked), (0, 1), "{report:?}");
        assert_eq!(backend.get_card("book", &id).unwrap().contents, renamed);

        let status = backend.status().unwrap();
        assert_eq!(status.parked.len(), 1, "{:?}", status.parked);
        assert_eq!(status.parked[0].kind, "remove");
    }

    /// A write over a conflicted card states no resolution, so the owner
    /// parks it (pimdir STORAGE §15.2, §15.3) rather than settle a
    /// conflict nobody decided: the card and its diverging body stay.
    #[test]
    fn a_write_over_a_conflicted_card_is_parked() {
        use io_pimdir::{
            change::PimdirWriteOp,
            collection::PimdirCollectionId,
            object::PimdirObject,
            placement::{PimdirBase, PimdirHandle, PimdirPlacement, PimdirStatus},
        };

        let (dir, mut backend) = store();
        let rename = |name: &str| {
            String::from_utf8(JANE.to_vec())
                .unwrap()
                .replace("Jane Doe", name)
                .into_bytes()
        };
        // The card as `source` holds it under `handle`, in sync with its
        // own server at `revision`.
        let pull = |source: &str, handle: &str, body: &[u8], revision: &str| {
            let mut store = io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_source(source);
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
                        collection: PimdirCollectionId::from("book"),
                        handle: PimdirHandle(handle.into()),
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
                            revision: Some(revision.into()),
                            object: Some(hash),
                        }),
                        origin: None,
                    }),
                ])
                .unwrap()
        };

        // NOTE: two sources hold the card and each pulls another edit of
        // it, so the second meets a divergence nobody decided.
        pull("phone", "1", JANE, "v1");
        pull("carddav", "jane.vcf", JANE, "e1");
        pull("phone", "1", &rename("Jane Roe"), "v2");
        let written = pull("carddav", "jane.vcf", &rename("Jane Poe"), "e2");
        assert_eq!(written.conflicted.len(), 1, "no conflict to write over");

        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        let read = backend.get_card("book", &id).unwrap();
        assert_eq!(read.contents, rename("Jane Roe"));

        backend
            .update_card("book", &id, rename("Jane Zoe"), read.etag.as_deref())
            .unwrap();
        backend
            .delete_card("book", &id, read.etag.as_deref())
            .unwrap();

        let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("carddav");
        let report = owner.drain().unwrap();
        assert_eq!((report.applied, report.parked), (0, 2), "{report:?}");
        assert_eq!(
            backend.get_card("book", &id).unwrap().contents,
            rename("Jane Roe")
        );

        let status = backend.status().unwrap();
        let parked: Vec<_> = status.parked.iter().map(|row| &row.kind).collect();
        assert_eq!(parked, ["update", "remove"]);
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
            let id = backend.list_cards(book, 0, None).unwrap().cards[0]
                .id
                .clone();
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
        assert_eq!(parked.kind, "update");
        assert!(
            parked.error.contains("body replaced since it was read"),
            "{}",
            parked.error
        );

        let text = status.to_string();
        let row = parked.row.to_string();
        let parked_line = [row.as_str(), "update", "work/book"];
        let mut lines = text.lines();
        let found = lines.any(|line| line.split_whitespace().take(3).eq(parked_line));
        assert!(found, "{text}");
        assert!(text.contains(&parked.error), "{text}");

        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["parked"][0]["addressbookId"], "work/book");
        assert_eq!(json["parked"][0]["row"], parked.row);
    }

    /// An update applied, then refused for good by the server it was
    /// pushed to, reads applied on its card with the refusal in `pimdir
    /// queue show`, and lists in `pimdir status` (pimdir STORAGE §15.4).
    #[test]
    fn a_refused_push_shows_on_its_row_and_in_the_status() {
        use io_pimdir::{
            change::{PimdirChangeKey, PimdirWriteOp},
            collection::PimdirCollectionId,
            placement::PimdirHandle,
        };

        let (dir, mut backend) = store();
        pulled(&dir, "book", JANE);
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        let renamed = String::from_utf8(JANE.to_vec())
            .unwrap()
            .replace("Jane Doe", "Jane Roe")
            .into_bytes();
        backend.update_card("book", &id, renamed, None).unwrap();
        drain(&dir);
        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .for_source("carddav")
            .write(vec![PimdirWriteOp::RefusePlacement {
                collection: PimdirCollectionId::from("book"),
                handle: PimdirHandle("jane.vcf".into()),
                key: PimdirChangeKey::from("892471aec94fc225"),
                reason: String::from("403 Forbidden"),
            }])
            .unwrap();

        // The update is the first row this store queued.
        let show = crate::pimdir::queue::show::PimdirQueueShowOutput::new(
            1,
            backend.queue_row(1).unwrap(),
        );
        let json = serde_json::to_value(&show).unwrap();
        assert_eq!(json["state"], "applied", "{json}");
        assert_eq!(json["seq"].to_string(), id, "{json}");
        assert_eq!(json["refusals"][0]["source"], "carddav", "{json}");
        assert_eq!(json["refusals"][0]["collection"], "book", "{json}");
        assert_eq!(json["refusals"][0]["reason"], "403 Forbidden", "{json}");
        assert!(
            show.to_string().contains("Refused by carddav in book at "),
            "{show}"
        );

        let status = backend.status().unwrap();
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["refused"][0]["addressbookId"], "book", "{json}");
        assert_eq!(json["refused"][0]["itemId"], id.as_str(), "{json}");
        assert_eq!(json["refused"][0]["source"], "carddav", "{json}");
        assert_eq!(json["refused"][0]["reason"], "403 Forbidden", "{json}");
        let text = status.to_string();
        let row = [id.as_str(), "book", "carddav", "403"];
        let found = text
            .lines()
            .any(|line| line.split_whitespace().take(4).eq(row));
        assert!(found, "{text}");
        assert!(text.contains("held until the card is edited"), "{text}");
    }

    /// A composed card refused as a duplicate keeps its composer file,
    /// named in the error, as any failed write does: the edit is not lost
    /// to a success that never was.
    #[cfg(unix)]
    #[test]
    fn a_composed_duplicate_keeps_its_file() {
        use clap::Parser;
        use pimalaya_cli::error::code_of;

        use crate::shared::card::add::CardAddCommand;

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        let jane = String::from_utf8(JANE.to_vec()).unwrap();
        let mut out = Captured::default();

        let err = CardAddCommand::parse_from([
            "add",
            "-k",
            "book",
            "-i",
            "--composer",
            // NOTE: a suffix glued to `-i` is what GNU and BSD sed both
            // read as in-place.
            "sed -i.bak s/Doe/Roe/",
            &jane,
        ])
        .execute(&mut out, &mut client(&dir))
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

    /// Rewrites, in the closed store at `root`, every occurrence of `from`
    /// into `to`, a string of the same length, so each record keeps its
    /// size. The store is opened and closed as its owner first, so its
    /// write-ahead log is checkpointed into pimdir.db and gone.
    fn rewrite(root: &std::path::Path, from: &str, to: &str) {
        assert_eq!(from.len(), to.len());
        drop(io_pimdir::client::PimdirStore::open(root).unwrap());
        assert!(!root.join("pimdir.db-wal").exists());

        let path = root.join("pimdir.db");
        let mut db = std::fs::read(&path).unwrap();
        let (from, to) = (from.as_bytes(), to.as_bytes());
        let mut found = false;
        for at in 0..=db.len() - from.len() {
            if &db[at..at + from.len()] == from {
                db[at..at + from.len()].copy_from_slice(to);
                found = true;
            }
        }
        assert!(found, "no {from:?} in the store");
        std::fs::write(&path, db).unwrap();
    }

    /// A store of another format version, here a draft-11 one an earlier
    /// neverest wrote, carries `store-version` and asks for a reinstall through
    /// neverest, the queue drained first, never for its removal.
    #[test]
    fn a_store_of_another_version_asks_for_a_reinstall() {
        use pimalaya_cli::error::code_of;

        let (dir, backend) = store();
        drop(backend);
        rewrite(dir.path(), FORMAT_VERSION, "storage-draft-11");

        let err = match PimdirBackend::new(PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        }) {
            Ok(_) => panic!("a store of another version opened"),
            Err(err) => err,
        };
        assert_eq!(code_of(&err), Some(ErrorCode::StoreVersion), "{err:#}");
        let message = format!("{err:#}");
        assert!(message.contains("storage-draft-11"), "{message}");
        assert!(message.contains(FORMAT_VERSION), "{message}");
        assert!(message.contains("move the store aside"), "{message}");
        assert!(message.contains("until nothing is queued"), "{message}");
        assert!(message.contains("neverest init"), "{message}");
        assert!(!message.contains("delete"), "{message}");
        assert!(!message.contains("sync a new one"), "{message}");
    }

    /// A store another user can read is refused before anything opens,
    /// with `store-exposed` (pimdir STORAGE §18).
    #[cfg(unix)]
    #[test]
    fn a_store_others_can_read_is_refused() {
        use std::os::unix::fs::PermissionsExt;

        use pimalaya_cli::error::code_of;

        let (dir, backend) = store();
        drop(backend);
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();

        let err = match PimdirBackend::new(PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        }) {
            Ok(_) => panic!("a store others can read opened"),
            Err(err) => err,
        };
        assert_eq!(code_of(&err), Some(ErrorCode::StoreExposed), "{err:#}");
    }

    /// A queued action whose payload misses its version, as only another
    /// program writes one, fails `pimdir status` with `action-malformed`
    /// rather than with no code, until the owner's drain parks it.
    #[test]
    fn a_malformed_queued_action_carries_its_code() {
        use pimalaya_cli::error::code_of;

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        drop(backend);
        rewrite(dir.path(), "\"v\":1", "\"w\":1");

        let mut backend = PimdirBackend::new(PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        })
        .unwrap();
        let err = backend.status().unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::ActionMalformed), "{err:#}");

        drain(&dir);
        let status = backend.status().unwrap();
        assert_eq!(status.parked.len(), 1);
    }

    /// A queued creation lists in the queue by its row, the link id the
    /// write reported and its full name, shows `pending`, and cancelling
    /// it drops it from the queue, after which its row reads `unknown`.
    #[test]
    fn a_queued_card_lists_in_the_queue_until_cancelled() {
        use io_pimdir::client::producer::PimdirActionStatus;

        let (_dir, mut backend) = store();
        let link_id = backend.create_card("book", JANE.to_vec()).unwrap();

        let queued = backend.queued_cards(None).unwrap();
        assert_eq!(queued.len(), 1, "{queued:?}");
        assert_eq!(queued[0].collection, "book");
        assert_eq!(queued[0].kind, "add");
        assert_eq!(queued[0].link_id.as_deref(), Some(link_id.as_str()));
        assert_eq!(queued[0].item_id, None);
        assert_eq!(queued[0].full_name.as_deref(), Some("Jane Doe"));
        assert_eq!((queued[0].attempts, queued[0].unconfirmed), (0, false));
        assert_eq!(queued[0].error, None);
        assert_eq!(backend.queued_cards(Some("book")).unwrap().len(), 1);

        let row = queued[0].id;
        let status = backend.queue_row(row).unwrap();
        assert!(
            matches!(&status, PimdirActionStatus::Pending { kind, .. } if kind == "add"),
            "{status:?}"
        );

        assert_eq!(backend.cancel_queued(row).unwrap(), "add");
        assert!(backend.queued_cards(None).unwrap().is_empty());
        assert_eq!(pending(&backend), 0);
        let gone = backend.queue_row(row).unwrap();
        assert!(matches!(gone, PimdirActionStatus::Unknown), "{gone:?}");

        let err = backend.cancel_queued(row).unwrap_err().to_string();
        assert!(err.contains("No queued action with row"), "{err}");
    }

    /// A row the sync applied reads `applied` with the card it created,
    /// the id `card read` takes, and is no longer cancelled.
    #[test]
    fn an_applied_creation_names_its_card_and_is_not_cancelled() {
        use io_pimdir::client::producer::PimdirActionStatus;

        let (dir, mut backend) = store();
        backend.create_card("book", JANE.to_vec()).unwrap();
        let row = backend.queued_cards(None).unwrap()[0].id;
        drain(&dir);

        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        match backend.queue_row(row).unwrap() {
            PimdirActionStatus::Applied { seq, .. } => {
                assert_eq!(seq.map(|seq| seq.to_string()), Some(id));
            }
            status => panic!("{status:?}"),
        }

        let err = backend.cancel_queued(row).unwrap_err().to_string();
        assert_eq!(err, format!("Row {row} was applied already"));
    }

    /// A parked row lists among the parked ones with its kind, in the
    /// address book named and in every one, and an unknown address book
    /// is refused.
    #[test]
    fn a_parked_row_lists_with_its_kind() {
        let (dir, mut backend) = store();
        pulled(&dir, "book", JANE);
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
            .id
            .clone();
        let read = backend.get_card("book", &id).unwrap();
        backend
            .delete_card("book", &id, read.etag.as_deref())
            .unwrap();
        let renamed = String::from_utf8(JANE.to_vec())
            .unwrap()
            .replace("Jane Doe", "Jane Poe")
            .into_bytes();
        pulled(&dir, "book", &renamed);
        drain(&dir);

        let parked = backend.parked_rows(None).unwrap();
        assert_eq!(parked.len(), 1, "{parked:?}");
        assert_eq!(parked[0].kind, "remove");
        assert_eq!(parked[0].collection, "book");
        assert!(!parked[0].unconfirmed);
        assert_eq!(parked[0].link_id, None);
        assert_eq!(parked[0].item_id.as_deref(), Some(id.as_str()));
        assert_eq!(parked[0].full_name.as_deref(), Some("Jane Poe"));
        assert!(parked[0].error.is_some(), "{parked:?}");
        assert_eq!(backend.parked_rows(Some("book")).unwrap().len(), 1);

        let err = backend.parked_rows(Some("nope")).unwrap_err();
        assert_eq!(
            pimalaya_cli::error::code_of::<ErrorCode>(&err),
            Some(ErrorCode::AddressbookNotFound),
        );

        assert_eq!(backend.cancel_queued(parked[0].id).unwrap(), "remove");
        assert!(backend.parked_rows(None).unwrap().is_empty());
    }

    /// A pending creation and a parked update print one row, the family's
    /// (pimalaya-cli's output.md, scenario "A pending creation and a parked
    /// update"): every key, the `add` with its link id and no card, the
    /// `update` with the card it updates and why it was parked.
    #[test]
    fn a_pending_creation_and_a_parked_update_share_the_row() {
        use crate::pimdir::queue::list::PimdirQueueListOutput;

        const ANN: &[u8] =
            b"BEGIN:VCARD\r\nVERSION:4.0\r\nUID:urn:uuid:c2\r\nFN:Ann Lee\r\nEND:VCARD\r\n";

        let (dir, mut backend) = store();
        pulled(&dir, "book", JANE);
        let id = backend.list_cards("book", 0, None).unwrap().cards[0]
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
        drain(&dir);
        let link_id = backend.create_card("book", ANN.to_vec()).unwrap();

        let output = PimdirQueueListOutput {
            account: String::from("perso"),
            addressbook: None,
            cards: backend.queued_cards(None).unwrap(),
            parked: backend.parked_rows(None).unwrap(),
        };
        let json = serde_json::to_value(&output).unwrap();
        let (add, update) = (&json["cards"][0], &json["parked"][0]);

        let keys = [
            "row",
            "kind",
            "collection",
            "linkId",
            "itemId",
            "producer",
            "queuedAt",
            "attempts",
            "unconfirmed",
            "error",
            "fullName",
        ];
        for row in [add, update] {
            let mut printed: Vec<&str> = row
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            printed.sort_unstable();
            let mut expected = keys.to_vec();
            expected.sort_unstable();
            assert_eq!(printed, expected, "{row}");
        }

        assert_eq!(add["kind"], "add", "{add}");
        assert_eq!(add["linkId"], link_id.as_str(), "{add}");
        assert_eq!(add["itemId"], serde_json::Value::Null, "{add}");
        assert_eq!(add["error"], serde_json::Value::Null, "{add}");
        assert_eq!(add["fullName"], "Ann Lee", "{add}");

        assert_eq!(update["kind"], "update", "{update}");
        assert_eq!(update["linkId"], serde_json::Value::Null, "{update}");
        assert_eq!(update["itemId"], id.as_str(), "{update}");
        assert!(update["error"].is_string(), "{update}");
        assert_eq!(update["fullName"], "Jane Roe", "{update}");
    }
}
