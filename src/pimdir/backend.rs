//! pimdir arm of the shared-API client: glue mapping the shared addressbook
//! and card operations onto a local [pimdir](https://github.com/pimalaya/pimdir)
//! store.
//!
//! Reads project [`io_pimdir`]'s client read API (`list_collections`,
//! `list_items_page_asc`, `get_item`, `count_items`) plus the blob store,
//! building cards from the stored `v: 1` summary (pimdir SPEC §13). A card
//! whose body is not local (`level < Full`) still lists; [`get_card`] reports
//! "body not fetched" rather than an error, the cue to sync.
//!
//! [`get_card`]: PimdirBackend::get_card
//!
//! Writes stage io-replica [`ReplicaMutation`]s through the store's mutate
//! seam (never raw SQL), so the next sync derives and pushes them. A write is
//! attributed to the client's configured source; it fails loudly when the
//! store was not synced as that source (no binding for the card), rather than
//! silently staging a change no sync will carry.

use anyhow::{Result, anyhow, bail};
use io_pimdir::PimdirItem;
use io_replica::{
    client::ReplicaStorage,
    collection::ReplicaCollectionId,
    coroutine::{ReplicaArg, ReplicaCoroutine, ReplicaCoroutineState, ReplicaYield},
    mutate::{ReplicaMutate, ReplicaMutation},
    object::ReplicaObject,
    placement::{ReplicaFlags, ReplicaHandle, ReplicaPlacement},
};

use crate::{
    config::PimdirConfig,
    pimdir::{
        card::{self, CardSummary},
        client::PimdirClient,
        hash::content_hash,
    },
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::Card,
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
            .inner
            .store
            .list_collections()?
            .into_iter()
            .filter(|collection| collection.kind.is_empty() || collection.kind == CARD_KIND)
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

    /// Declares a local contact collection named `name`. The collection is
    /// local until a sync carries it: pimdir stages item mutations, not
    /// collection ones, so no remote address book is created here.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        _description: Option<&str>,
        _color: Option<&str>,
    ) -> Result<String> {
        if name.is_empty() {
            bail!("Addressbook name cannot be empty");
        }

        self.inner.store.ensure_collection(name, CARD_KIND)?;
        Ok(name.to_string())
    }

    /// Renames the collection identified by `id`.
    ///
    /// pimdir stores no display name, description or colour of its own beyond
    /// the collection row a sync writes, so only a rename is honoured: a patch
    /// carrying just a description or a colour is rejected rather than
    /// silently dropped.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        if patch.description.is_some() || patch.color.is_some() {
            bail!(
                "The pimdir backend stores no addressbook description or color; \
                 only a rename is supported"
            );
        }

        let Some(name) = patch.name else {
            return Ok(());
        };

        self.inner.store.rename_collection(id, &name)?;
        Ok(())
    }

    /// Always fails: io-pimdir exposes no collection removal, and io-replica
    /// has no collection-level mutation to stage one either, so a delete here
    /// would be a silent no-op.
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
        let seq = parse_id(card_id)?;
        let Some(item) = self.inner.store.get_item(addressbook_id, seq)? else {
            bail!("Card `{card_id}` not found in `{addressbook_id}`");
        };
        let Some(hash) = item.object.clone() else {
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

    /// Adds a locally-authored card to `addressbook_id`, staged as `Add` (the
    /// next sync uploads it). Returns the public id the store assigned.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let (link_id, meta, sort_key) = card::derive(&contents);
        let object = ReplicaObject {
            hash: content_hash(&contents),
            size: contents.len(),
        };
        let handle = ReplicaHandle(format!("local:{}", link_id.0));
        let link = link_id.0.clone();

        self.run_mutation(
            addressbook_id,
            ReplicaMutation::Add {
                handle,
                link_id,
                flags: ReplicaFlags::default(),
                object,
                body: contents,
                meta: Some(meta),
                sort_key,
            },
        )?;

        let seq = self
            .inner
            .store
            .seq_for_link(addressbook_id, &link)?
            .ok_or_else(|| anyhow!("Added card `{link}` in `{addressbook_id}` has no public id"))?;

        Ok(seq.to_string())
    }

    /// Replaces `card_id`'s body, staged as `Edit` (the next sync pushes it,
    /// three-way merging against the stored base).
    ///
    /// `if_match` is ignored: a staged edit is reconciled by the engine
    /// against the base body it recorded at sync time, which is a stronger
    /// guarantee than an ETag precondition a local store cannot check.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        _if_match: Option<&str>,
    ) -> Result<()> {
        let placement = self.synced_placement(addressbook_id, card_id)?;
        let (_, meta, sort_key) = card::derive(&contents);
        let object = ReplicaObject {
            hash: content_hash(&contents),
            size: contents.len(),
        };

        self.run_mutation(
            addressbook_id,
            ReplicaMutation::Edit {
                handle: placement.handle,
                object,
                body: contents,
                meta: Some(meta),
                sort_key: Some(sort_key),
            },
        )
    }

    /// Deletes `card_id` from `addressbook_id`, staged as `Remove` (a
    /// tombstone the next sync pushes as a server-side delete).
    pub fn delete_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<()> {
        let placement = self.synced_placement(addressbook_id, card_id)?;
        self.run_mutation(addressbook_id, ReplicaMutation::Remove(placement.handle))
    }

    /// Pulls every live item of a collection by keyset paging, in the
    /// contacts order the store maintains (display name ascending).
    fn scan_items(&self, addressbook_id: &str) -> Result<Vec<PimdirItem>> {
        let mut all: Vec<PimdirItem> = Vec::new();
        let mut cursor: Option<(String, i64)> = None;

        loop {
            let page = self.inner.store.list_items_page_asc(
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
        let contents = match &item.object {
            Some(hash) => self.inner.blobs.get(hash)?.unwrap_or_default(),
            None => preview_vcard(&summary_of(&item)),
        };

        Ok(Card {
            id: item.seq.to_string(),
            addressbook_id: addressbook_id.to_string(),
            etag: item.object.map(|hash| hash.0),
            contents,
        })
    }

    /// The source's placement for the public id `card_id`, guaranteed to carry
    /// a sync base.
    ///
    /// Resolves the public `seq` to the internal `link_id` first, then finds
    /// the placement by link id. A change on a placement with no base would
    /// stage as a fresh create rather than an edit and no sync would carry it,
    /// so this is the guard that turns a misconfigured source (the store was
    /// never synced as `self.inner.source`) into a clear error instead of a
    /// silent no-op.
    fn synced_placement(&self, collection: &str, card_id: &str) -> Result<ReplicaPlacement> {
        let seq = parse_id(card_id)?;
        let link_id = self
            .inner
            .store
            .get_item(collection, seq)?
            .map(|item| item.link_id.0)
            .ok_or_else(|| anyhow!("Card `{card_id}` not found in `{collection}`"))?;

        let loaded = self
            .inner
            .store
            .load(&ReplicaCollectionId(collection.to_string()))?;
        let placement = loaded
            .placements
            .into_iter()
            .find(|placement| {
                placement.link_id.as_ref().map(|link| link.0.as_str()) == Some(link_id.as_str())
            })
            .ok_or_else(|| anyhow!("Card `{card_id}` not found in `{collection}`"))?;

        if placement.base.is_none() {
            bail!(
                "`{collection}` was not synced as source `{}`, so `{card_id}` cannot be \
                 edited here; set `pimdir.source` to the sync source and sync first",
                self.inner.source
            );
        }

        Ok(placement)
    }

    /// Drives a mutate coroutine to completion against the store: it only ever
    /// asks to load the collection and to write the staged ops.
    fn run_mutation(&mut self, collection: &str, mutation: ReplicaMutation) -> Result<()> {
        let mut coroutine = ReplicaMutate::new(collection.to_string(), mutation);
        let mut arg: Option<ReplicaArg> = None;

        loop {
            match coroutine.resume(arg.take()) {
                ReplicaCoroutineState::Yielded(ReplicaYield::WantsLoad(collection)) => {
                    let loaded = self.inner.store.load(&collection)?;
                    arg = Some(ReplicaArg::Load(loaded));
                }
                ReplicaCoroutineState::Yielded(ReplicaYield::WantsWrite(ops)) => {
                    self.inner.store.write(ops)?;
                    arg = Some(ReplicaArg::Write);
                }
                ReplicaCoroutineState::Yielded(_) => {
                    bail!("pimdir mutate asked for an unexpected step");
                }
                ReplicaCoroutineState::Complete(result) => {
                    return result.map_err(|err| anyhow!("pimdir mutate failed: {err}"));
                }
            }
        }
    }
}

/// Reads a stored item's `v: 1` summary, falling back to an empty one when the
/// card was never projected or the blob does not parse.
fn summary_of(item: &PimdirItem) -> CardSummary {
    item.meta
        .as_ref()
        .and_then(|meta| serde_json::from_str(&meta.0).ok())
        .unwrap_or_default()
}

/// Renders a stored summary as a minimal vCard, the listing preview of a card
/// whose body is not local yet. It carries only what the summary knows (`UID`,
/// `FN`, `EMAIL`), so a contact list reads correctly before a full sync.
fn preview_vcard(summary: &CardSummary) -> Vec<u8> {
    let mut out = String::from("BEGIN:VCARD\r\nVERSION:4.0\r\n");

    if let Some(uid) = &summary.uid {
        out.push_str(&format!("UID:{uid}\r\n"));
    }
    out.push_str(&format!("FN:{}\r\n", summary.full_name));
    for email in &summary.emails {
        out.push_str(&format!("EMAIL:{email}\r\n"));
    }
    out.push_str("END:VCARD\r\n");

    out.into_bytes()
}

/// Parses a card id, the public per-collection `seq` (a small integer), with a
/// clear error for a non-numeric one.
fn parse_id(id: &str) -> Result<i64> {
    id.parse::<i64>()
        .map_err(|_| anyhow!("Invalid card id `{id}` (expected a number)"))
}
