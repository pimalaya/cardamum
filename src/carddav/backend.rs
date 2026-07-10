//! CardDAV arm of the shared-API client: thin glue mapping the shared
//! addressbook and card operations onto
//! [`io_webdav::client::WebdavClientStd`] calls (RFC 6352).

use anyhow::{Result, bail};
use io_webdav::{
    client::WebdavClientStd,
    rfc6352::{addressbook::Addressbook as WireAddressbook, card::CardEntry},
};

use crate::{
    config::CarddavConfig,
    shared::{
        addressbook::{Addressbook, AddressbookDiff},
        card::Card,
        client::paginate,
    },
};

/// CardDAV backend of the shared-API client, wrapping a connected
/// io-webdav client with its addressbook home-set resolved.
pub struct CarddavBackend {
    pub inner: WebdavClientStd,
}

impl CarddavBackend {
    /// Opens the client from the account's `[carddav]` block, running
    /// discovery when needed (see
    /// [`open_carddav_client`](crate::carddav::client::open_carddav_client)).
    pub fn new(config: CarddavConfig) -> Result<Self> {
        let inner = crate::carddav::client::open_carddav_client(config)?;
        Ok(Self { inner })
    }

    /// Lists every addressbook under the discovered home-set.
    pub fn list_addressbooks(&mut self) -> Result<Vec<Addressbook>> {
        let addressbooks = self.inner.list_addressbooks()?;
        Ok(addressbooks.into_iter().map(into_addressbook).collect())
    }

    /// Creates an addressbook collection named `name` (also used as its
    /// URL segment) under the home-set. Returns the new addressbook id.
    pub fn create_addressbook(
        &mut self,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        if name.is_empty() {
            bail!("Addressbook name cannot be empty");
        }

        let wire = WireAddressbook {
            id: name.to_string(),
            display_name: Some(name.to_string()),
            description: description.map(str::to_string),
            color: color.map(str::to_string),
            ctag: None,
            sync_token: None,
        };
        self.inner.create_addressbook(&wire)?;

        Ok(name.to_string())
    }

    /// Applies `patch` to the addressbook identified by `id`, merging
    /// it against the current collection properties.
    pub fn update_addressbook(&mut self, id: &str, patch: AddressbookDiff) -> Result<()> {
        let addressbooks = self.inner.list_addressbooks()?;
        let current = addressbooks
            .into_iter()
            .find(|addressbook| addressbook.id == id)
            .ok_or_else(|| anyhow::anyhow!("Addressbook `{id}` not found"))?;

        let next = WireAddressbook {
            id: id.to_string(),
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
            ctag: None,
            sync_token: None,
        };

        self.inner.update_addressbook(&next)?;
        Ok(())
    }

    /// Deletes the addressbook collection identified by `id`.
    pub fn delete_addressbook(&mut self, id: &str) -> Result<()> {
        self.inner.delete_addressbook(id)?;
        Ok(())
    }

    /// Lists the cards inside `addressbook_id`, applying 1-indexed
    /// pagination.
    pub fn list_cards(
        &mut self,
        addressbook_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<Card>> {
        let entries = self.inner.list_cards(addressbook_id)?;
        let cards = entries
            .into_iter()
            .map(|entry| into_card(addressbook_id, entry))
            .collect();
        Ok(paginate(cards, page, page_size))
    }

    /// Fetches `card_id` from `addressbook_id`.
    pub fn get_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<Card> {
        let body = self.inner.read_card(addressbook_id, card_id)?;

        Ok(Card {
            id: card_id.to_string(),
            addressbook_id: addressbook_id.to_string(),
            etag: body.etag,
            contents: body.data,
        })
    }

    /// Creates a new card inside `addressbook_id` under a fresh UUID
    /// resource name. Returns the assigned id.
    pub fn create_card(&mut self, addressbook_id: &str, contents: Vec<u8>) -> Result<String> {
        let id = fresh_card_id()?;
        let created = self.inner.create_card(addressbook_id, &id, contents)?;
        Ok(created.id)
    }

    /// Overwrites `card_id` inside `addressbook_id`, gating on
    /// `if_match` when present (RFC 9110 If-Match).
    pub fn update_card(
        &mut self,
        addressbook_id: &str,
        card_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.inner
            .update_card(addressbook_id, card_id, contents, if_match)?;
        Ok(())
    }

    /// Permanently deletes `card_id` from `addressbook_id`.
    pub fn delete_card(&mut self, addressbook_id: &str, card_id: &str) -> Result<()> {
        self.inner.delete_card(addressbook_id, card_id, None)?;
        Ok(())
    }
}

/// Maps a WebDAV wire addressbook to the shared shape: the display
/// name falls back to the id.
fn into_addressbook(wire: WireAddressbook) -> Addressbook {
    let name = wire.display_name.unwrap_or_else(|| wire.id.clone());

    Addressbook {
        id: wire.id,
        name,
        description: wire.description,
        color: wire.color,
    }
}

/// Maps a WebDAV [`CardEntry`] to a shared [`Card`].
fn into_card(addressbook_id: &str, entry: CardEntry) -> Card {
    Card {
        id: entry.id,
        addressbook_id: addressbook_id.to_string(),
        etag: entry.etag,
        contents: entry.data,
    }
}

/// Generates a fresh CardDAV card id (a random UUIDv4) from the system
/// entropy source; CardDAV requires the caller to name the resource.
fn fresh_card_id() -> Result<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|err| anyhow::anyhow!("Gather randomness error: {err}"))?;

    // NOTE: RFC 4122 4.4 stamps version 4 and variant 10xx.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = [0u8; 36];
    let mut cursor = 0;
    for (i, byte) in bytes.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            out[cursor] = b'-';
            cursor += 1;
        }
        out[cursor] = HEX[(byte >> 4) as usize];
        out[cursor + 1] = HEX[(byte & 0x0f) as usize];
        cursor += 2;
    }

    Ok(String::from_utf8(out.to_vec()).expect("ASCII hex is always valid UTF-8"))
}
