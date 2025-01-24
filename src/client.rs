use addressbook::{Addressbook, Addressbooks, Card, Cards, PartialAddressbook};
use color_eyre::{eyre::bail, Result};

use crate::account::config::Backend;
#[cfg(feature = "_carddav")]
use crate::carddav;
#[cfg(feature = "_vdir")]
use crate::vdir;

pub enum Client {
    None,
    #[cfg(feature = "_carddav")]
    CardDav(carddav::Client),
    #[cfg(feature = "_vdir")]
    Vdir(vdir::Client),
}

impl Client {
    pub fn new(backend: Backend) -> Result<Self> {
        Ok(match backend {
            #[cfg(feature = "_carddav")]
            Backend::CardDav(config) => Self::CardDav(carddav::Client::new(config)?),
            #[cfg(feature = "_vdir")]
            Backend::Vdir(config) => Self::Vdir(vdir::Client::new(config)),
            Backend::None => Self::None,
        })
    }

    pub fn create_addressbook(&self, addressbook: Addressbook) -> Result<Addressbook> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.create_addressbook(addressbook),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.create_addressbook(addressbook),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn list_addressbooks(&self) -> Result<Addressbooks> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.list_addressbooks(),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.list_addressbooks(),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn list_cards(&self, addressbook_id: impl AsRef<str>) -> Result<Cards> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.list_cards(addressbook_id),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.list_cards(addressbook_id),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn update_addressbook(
        &self,
        addressbook: PartialAddressbook,
    ) -> Result<PartialAddressbook> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.update_addressbook(addressbook),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.update_addressbook(addressbook),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn delete_addressbook(&self, id: impl AsRef<str>) -> Result<bool> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.delete_addressbook(id),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.delete_addressbook(id),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn create_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.create_card(addressbook_id, card),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.create_card(addressbook_id, card),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn read_card(
        &self,
        addressbook_id: impl AsRef<str>,
        card_id: impl ToString,
    ) -> Result<Card> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.read_card(addressbook_id, card_id),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.read_card(addressbook_id, card_id),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn update_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.update_card(addressbook_id, card),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.update_card(addressbook_id, card),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn delete_card(
        &self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<()> {
        match self {
            #[cfg(feature = "_carddav")]
            Self::CardDav(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "_vdir")]
            Self::Vdir(client) => client.delete_card(addressbook_id, card_id),
            Self::None => bail!("client not defined"),
        }
    }
}
