//! # Vdir backend
//!
//! Vdir arm of the unified-API client, mapping the unified addressbook and
//! card operations onto [`io_vdir::client::VdirClient`] calls against the
//! configured home directory.

use std::io;

use anyhow::{Error, Result, bail};
use io_vdir::{
    client::{VdirClient, VdirClientError},
    collection::VdirCollection,
    item::VdirItemKind,
    path::VdirPath,
};
use pimalaya_cli::error::CodedError;

use crate::{
    config::VdirConfig,
    error::{self, ErrorCode},
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::{Card, CardUpdateOutcome},
        client::{CardWindow, first_page, window},
    },
    vdir::client::{check_collection_absent, collection_dir, is_dir_name, missing_code},
};

/// Vdir backend of the unified-API client, rooted at the home directory.
pub struct VdirBackend {
    pub inner: VdirClient,
}

impl VdirBackend {
    /// Opens the backend against the configured home directory.
    pub fn new(config: VdirConfig) -> Self {
        Self {
            inner: VdirClient::new(config.root),
        }
    }

    /// Lists every addressbook (collection) under the home directory.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let collections = self.inner.list_collections()?;
        Ok(collections.into_iter().map(into_addressbook).collect())
    }

    /// Creates a collection named after `name` under the home directory.
    ///
    /// Returns the new addressbook id, which is its directory name. A name
    /// that is not one directory name is refused, as it would reach out of
    /// the home or below one of its addressbooks, and so is one the home
    /// holds already.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if name.is_empty() {
            bail!("Addressbook name cannot be empty");
        }

        if !is_dir_name(name) {
            bail!("Addressbook name `{name}` is not one directory name");
        }

        check_collection_absent(self.inner.root(), name)?;

        let collection = VdirCollection {
            path: self.inner.root().join(name),
            display_name: Some(name.to_string()),
            description: description.map(str::to_string),
            color: color.map(str::to_string),
        };
        self.inner.create_collection(collection)?;

        Ok(name.to_string())
    }

    /// Applies `patch` to the collection `id`, over its current metadata.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        let path = self.addressbook_path(id)?;
        let collections = self.inner.list_collections()?;
        let current = collections
            .into_iter()
            .find(|collection| collection.id() == id)
            .ok_or_else(|| addressbook_not_found(id))?;

        let next = VdirCollection {
            path,
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

    /// Lists the `take` vCard items inside `addressbook_id` after the first
    /// `skip`.
    ///
    /// A file that cannot be read is an entry naming its card, on the
    /// first page only, the whole directory being read for every page; a
    /// file removed while it lists is left out.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        skip: usize,
        take: Option<usize>,
    ) -> Result<CardWindow> {
        let path = self.addressbook_path(addressbook_id)?;
        let (items, unreadable) = self.inner.list_readable_items(path)?;

        let errors = unreadable
            .into_iter()
            .filter(|(entry, _)| entry.kind == VdirItemKind::Vcard)
            .filter(|(_, err)| err.kind() != io::ErrorKind::NotFound)
            .map(|(entry, err)| {
                let err = Error::new(err).context("Cannot read the card file");
                error::unreadable_card(Some(&entry.id), err)
            })
            .collect();

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

        Ok(CardWindow {
            cards: window(cards, skip, take),
            errors: first_page(errors, skip),
        })
    }

    /// Fetches `card_id` from `addressbook_id`.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let path = self.addressbook_path(addressbook_id)?;
        let item = self
            .inner
            .get_item(path, card_id)
            .map_err(|err| card_error(addressbook_id, card_id, err))?;

        Ok(Card {
            id: card_id.to_string(),
            addressbook_id: addressbook_id.to_string(),
            etag: None,
            contents: item.contents,
        })
    }

    /// Stores a new vCard inside `addressbook_id`, returning its id.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let path = self.addressbook_path(addressbook_id)?;
        let (id, _) = self
            .inner
            .store_item(path, None, VdirItemKind::Vcard, contents)?;
        Ok(id)
    }

    /// Overwrites `card_id` inside `addressbook_id`.
    ///
    /// The item is located first: storing by id writes whatever file name it
    /// is given, so an unknown id would be created rather than rejected.
    /// `if_match` is refused, vdir having no entity tag to check it against.
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<CardUpdateOutcome> {
        if if_match.is_some() {
            bail!("The vdir backend has no ETag, so it cannot honour If-Match");
        }

        let path = self.addressbook_path(addressbook_id)?;
        self.inner
            .locate_item(path.clone(), card_id)
            .map_err(|err| card_error(addressbook_id, card_id, err))?;

        self.inner.store_item(
            path,
            Some(card_id.to_string()),
            VdirItemKind::Vcard,
            contents,
        )?;
        Ok(CardUpdateOutcome::default())
    }

    /// Permanently deletes `card_id` from `addressbook_id`.
    pub fn delete_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        if if_match.is_some() {
            bail!("The vdir backend has no ETag, so it cannot honour If-Match");
        }
        let path = self.addressbook_path(addressbook_id)?;
        self.inner
            .delete_item(path, card_id)
            .map_err(|err| card_error(addressbook_id, card_id, err))?;
        Ok(())
    }

    /// Resolves `addressbook_id` to an existing collection directory.
    ///
    /// A write creates whatever directory it is handed, a read of a missing
    /// one lists nothing and a delete removes whatever the path reaches, so
    /// a path built without looking would let a typo in `-k` invent an
    /// addressbook, then list it as empty, and an id that is not one
    /// directory name (`..`, `../keep`) remove a directory out of the home.
    fn addressbook_path(&self, addressbook_id: &str) -> Result<VdirPath> {
        collection_dir(self.inner.root(), addressbook_id)
            .ok_or_else(|| addressbook_not_found(addressbook_id))
    }
}

/// The failure of a request on the card `card_id`, `card-not-found` when
/// the address book holds no such card, io-vdir's text in `sources`.
fn card_error(addressbook_id: &str, card_id: &str, err: VdirClientError) -> Error {
    match missing_code(&err) {
        Some(ErrorCode::CardNotFound) => error::card_not_found(addressbook_id, card_id, err.into()),
        _ => err.into(),
    }
}

/// The failure of the address book `id` the home directory does not hold.
fn addressbook_not_found(id: &str) -> Error {
    CodedError::new(
        ErrorCode::AddressbookNotFound,
        format!("Addressbook `{id}` not found"),
    )
    .into()
}

/// Maps a [`VdirCollection`] to a shared [`Addressbook`].
///
/// The final path segment is the id, which the display name falls back to.
fn into_addressbook(collection: VdirCollection) -> Addressbook {
    let id = collection.id().to_string();
    let name = collection.display_name.unwrap_or_else(|| id.clone());

    Addressbook {
        id,
        name,
        description: collection.description,
        color: collection.color,
        default: false,
    }
}
