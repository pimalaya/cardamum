use std::collections::HashSet;

use anyhow::{anyhow, bail, Result};
use io_addressbook::Addressbook;

use crate::account::Account;
#[cfg(feature = "carddav")]
use crate::carddav::client::Client as CarddavClient;

#[derive(Debug, Default)]
pub enum Client<'a> {
    #[default]
    None,
    #[cfg(feature = "carddav")]
    Carddav(CarddavClient<'a>),
    #[cfg(feature = "vdir")]
    Vdir(crate::vdir::Client),
}

impl<'a> Client<'a> {
    pub fn new(account: &'a Account) -> Result<Self> {
        #[cfg(feature = "carddav")]
        if let Some(config) = &account.carddav {
            return Ok(Self::Carddav(CarddavClient::new(config)?));
        }

        #[cfg(feature = "vdir")]
        if scheme == "file" || scheme == "" {
            return Ok(Self::Vdir(VdirClient::new(account)?));
        }

        Err(anyhow!("Cannot find CardDAV nor Vdir config")
            .context("Create addressbook client error"))
    }

    // pub fn create_addressbook(&self, addressbook: Addressbook) -> Result<Addressbook> {
    //     match self {
    //         #[cfg(feature = "carddav")]
    //         Self::CardDav(client) => client.create_addressbook(addressbook),
    //         #[cfg(feature = "vdir")]
    //         Self::Vdir(client) => client.create_addressbook(addressbook),
    //         Self::None => bail!("client not defined"),
    //     }
    // }

    pub fn list_addressbooks(&mut self) -> Result<HashSet<Addressbook>> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.list_addressbooks(),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.list_addressbooks(),
        }
    }

    // pub fn list_cards(&self, addressbook_id: impl AsRef<str>) -> Result<HashSet<Card>> {
    //     match self {
    //         #[cfg(feature = "carddav")]
    //         Self::CardDav(client) => client.list_cards(addressbook_id),
    //         #[cfg(feature = "vdir")]
    //         Self::Vdir(client) => client.list_cards(addressbook_id),
    //         Self::None => bail!("client not defined"),
    //     }
    // }

    // pub fn update_addressbook(&self, addressbook: Addressbook) -> Result<()> {
    //     match self {
    //         #[cfg(feature = "carddav")]
    //         Self::CardDav(client) => client.update_addressbook(addressbook),
    //         #[cfg(feature = "vdir")]
    //         Self::Vdir(client) => client.update_addressbook(addressbook),
    //         Self::None => bail!("client not defined"),
    //     }
    // }

    // pub fn delete_addressbook(&self, id: impl AsRef<str>) -> Result<bool> {
    //     match self {
    //         #[cfg(feature = "carddav")]
    //         Self::CardDav(client) => client.delete_addressbook(id),
    //         #[cfg(feature = "vdir")]
    //         Self::Vdir(client) => client.delete_addressbook(id),
    //         Self::None => bail!("client not defined"),
    //     }
    // }

    // pub fn create_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
    //     match self {
    //         #[cfg(feature = "carddav")]
    //         Self::CardDav(client) => client.create_card(addressbook_id, card),
    //         #[cfg(feature = "vdir")]
    //         Self::Vdir(client) => client.create_card(addressbook_id, card),
    //         Self::None => bail!("client not defined"),
    //     }
    // }

    // pub fn read_card(
    //     &self,
    //     addressbook_id: impl AsRef<str>,
    //     card_id: impl ToString,
    // ) -> Result<Card> {
    //     match self {
    //         #[cfg(feature = "carddav")]
    //         Self::CardDav(client) => client.read_card(addressbook_id, card_id),
    //         #[cfg(feature = "vdir")]
    //         Self::Vdir(client) => client.read_card(addressbook_id, card_id),
    //         Self::None => bail!("client not defined"),
    //     }
    // }

    // pub fn update_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
    //     match self {
    //         #[cfg(feature = "carddav")]
    //         Self::CardDav(client) => client.update_card(addressbook_id, card),
    //         #[cfg(feature = "vdir")]
    //         Self::Vdir(client) => client.update_card(addressbook_id, card),
    //         Self::None => bail!("client not defined"),
    //     }
    // }

    // pub fn delete_card(
    //     &self,
    //     addressbook_id: impl AsRef<str>,
    //     card_id: impl AsRef<str>,
    // ) -> Result<()> {
    //     match self {
    //         #[cfg(feature = "carddav")]
    //         Self::CardDav(client) => client.delete_card(addressbook_id, card_id),
    //         #[cfg(feature = "vdir")]
    //         Self::Vdir(client) => client.delete_card(addressbook_id, card_id),
    //         Self::None => bail!("client not defined"),
    //     }
    // }
}
