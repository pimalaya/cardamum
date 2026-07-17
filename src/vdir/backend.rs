//! Vdir arm of the shared-API client: thin glue mapping the shared
//! addressbook and card operations onto [`io_vdir::client::VdirClient`]
//! calls against the configured home directory.

use anyhow::{Result, bail};
use io_vdir::{client::VdirClient, collection::VdirCollection, item::VdirItemKind, path::VdirPath};

use crate::{
    config::VdirConfig,
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::Card,
        client::paginate,
    },
};

/// Vdir backend of the shared-API client, wrapping the io-vdir
/// filesystem client rooted at the account's home directory.
pub struct VdirBackend {
    pub inner: VdirClient,
}

impl VdirBackend {
    pub fn new(config: VdirConfig) -> Self {
        Self {
            inner: VdirClient::new(config.home_dir),
        }
    }

    /// Lists every addressbook (collection) under the home directory.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let collections = self.inner.list_collections()?;
        Ok(collections.into_iter().map(into_addressbook).collect())
    }

    /// Creates a collection named after `name` under the home
    /// directory. Returns the new addressbook id (its directory name).
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if name.is_empty() {
            bail!("Addressbook name cannot be empty");
        }

        let collection = VdirCollection {
            path: self.inner.root().join(name),
            display_name: Some(name.to_string()),
            description: description.map(str::to_string),
            color: color.map(str::to_string),
        };
        self.inner.create_collection(collection)?;

        Ok(name.to_string())
    }

    /// Applies `patch` to the collection identified by `id`, merging it
    /// against the current collection metadata.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        let collections = self.inner.list_collections()?;
        let current = collections
            .into_iter()
            .find(|collection| collection.id() == id)
            .ok_or_else(|| anyhow::anyhow!("Addressbook `{id}` not found"))?;

        let next = VdirCollection {
            path: self.addressbook_path(id)?,
            display_name: match patch.name {
                Some(name) => Some(name),
                None => current.display_name,
            },
            description: match patch.description {
                Some(description) => description,
                None => current.description,
            },
            color: match patch.color {
                Some(color) => color,
                None => current.color,
            },
        };

        self.inner.update_collection(next)?;
        Ok(())
    }

    /// Recursively removes the collection identified by `id`.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        let path = self.addressbook_path(id)?;
        self.inner.delete_collection(path)?;
        Ok(())
    }

    /// Lists the vCard items inside `addressbook_id`, applying
    /// 1-indexed pagination.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        let path = self.addressbook_path(addressbook_id)?;
        let items = self.inner.list_items(path)?;

        let cards = items
            .into_iter()
            .filter(|item| item.kind == VdirItemKind::Vcard)
            .filter_map(|item| {
                let id = item.id()?.to_string();
                Some(Card {
                    id,
                    addressbook_id: addressbook_id.to_string(),
                    etag: None,
                    contents: item.contents,
                })
            })
            .collect();

        Ok(paginate(cards, page, page_size))
    }

    /// Fetches `card_id` from `addressbook_id`.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let path = self.addressbook_path(addressbook_id)?;
        let item = self.inner.get_item(path, card_id)?;

        Ok(Card {
            id: card_id.to_string(),
            addressbook_id: addressbook_id.to_string(),
            etag: None,
            contents: item.contents,
        })
    }

    /// Stores a new vCard inside `addressbook_id`. Returns its assigned
    /// id.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let path = self.addressbook_path(addressbook_id)?;
        let (id, _) = self
            .inner
            .store_item(path, None, VdirItemKind::Vcard, contents)?;
        Ok(id)
    }

    /// Overwrites `card_id` inside `addressbook_id`. `if_match` is
    /// ignored: vdir has no entity-tag concept.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        _if_match: Option<&str>,
    ) -> Result<()> {
        let path = self.addressbook_path(addressbook_id)?;
        self.inner.store_item(
            path,
            Some(card_id.to_string()),
            VdirItemKind::Vcard,
            contents,
        )?;
        Ok(())
    }

    /// Permanently deletes `card_id` from `addressbook_id`.
    pub fn delete_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<()> {
        let path = self.addressbook_path(addressbook_id)?;
        self.inner.delete_item(path, card_id)?;
        Ok(())
    }

    /// Resolves `addressbook_id` against the home directory, rejecting
    /// an empty id.
    fn addressbook_path(&self, addressbook_id: &str) -> Result<VdirPath> {
        if addressbook_id.is_empty() {
            bail!("Addressbook id cannot be empty");
        }
        Ok(self.inner.root().join(addressbook_id))
    }
}

/// Maps a vdir [`VdirCollection`] to a shared [`Addressbook`]: the final
/// path segment is the id and the display name falls back to it.
fn into_addressbook(collection: VdirCollection) -> Addressbook {
    let id = collection.id().to_string();
    let name = collection.display_name.unwrap_or_else(|| id.clone());

    Addressbook {
        id,
        name,
        description: collection.description,
        color: collection.color,
    }
}
