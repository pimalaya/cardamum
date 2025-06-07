use std::collections::HashSet;

use anyhow::{bail, Result};
use io_addressbook::Addressbook;

use crate::account::Account;
#[cfg(feature = "carddav")]
use crate::carddav::client::Client as CarddavClient;

#[derive(Debug, Default)]
pub enum Client {
    #[default]
    None,
    #[cfg(feature = "carddav")]
    Carddav(CarddavClient),
    #[cfg(feature = "vdir")]
    Vdir(crate::vdir::Client),
}

impl Client {
    pub fn new(account: Account) -> Result<Self> {
        match account {
            Account::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Account::Carddav(config) => Ok(Self::Carddav(CarddavClient::new(config)?)),
            #[cfg(feature = "vdir")]
            Account::Vdir(config) => Ok(Self::CardDav(crate::vdir::Client::new(account)?)),
        }
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
