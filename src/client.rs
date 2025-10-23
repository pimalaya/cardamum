use std::collections::HashSet;

use anyhow::{anyhow, bail, Result};
use io_addressbook::{addressbook::Addressbook, card::Card};

use crate::account::Account;
#[cfg(feature = "carddav")]
use crate::carddav::client::CarddavClient;
#[cfg(feature = "vdir")]
use crate::vdir::client::VdirClient;

#[derive(Debug, Default)]
pub enum Client<'a> {
    #[default]
    None,
    #[cfg(feature = "carddav")]
    Carddav(CarddavClient<'a>),
    #[cfg(feature = "vdir")]
    Vdir(VdirClient),
}

impl<'a> Client<'a> {
    pub fn new(account: &'a Account) -> Result<Self> {
        #[cfg(feature = "carddav")]
        if let Some(config) = &account.carddav {
            return Ok(Self::Carddav(CarddavClient::new(config)?));
        }

        #[cfg(feature = "vdir")]
        if let Some(config) = &account.vdir {
            return Ok(Self::Vdir(VdirClient::new(config)));
        }

        Err(anyhow!("Cannot find CardDAV nor Vdir config")
            .context("Create addressbook client error"))
    }

    pub fn create_addressbook(&mut self, addressbook: Addressbook) -> Result<()> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.create_addressbook(addressbook),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.create_addressbook(addressbook),
        }
    }

    pub fn list_addressbooks(&mut self) -> Result<HashSet<Addressbook>> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.list_addressbooks(),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.list_addressbooks(),
        }
    }

    pub fn list_cards(&mut self, addressbook_id: impl AsRef<str>) -> Result<HashSet<Card>> {
        match self {
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.list_cards(addressbook_id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.list_cards(addressbook_id),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn update_addressbook(&mut self, addressbook: Addressbook) -> Result<()> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.update_addressbook(addressbook),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.update_addressbook(addressbook),
        }
    }

    pub fn delete_addressbook(&mut self, id: impl AsRef<str>) -> Result<()> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.delete_addressbook(id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.delete_addressbook(id),
        }
    }

    pub fn create_card(&mut self, card: Card) -> Result<()> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.create_card(card),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.create_card(card),
        }
    }

    pub fn read_card(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<Card> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.read_card(addressbook_id, card_id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.read_card(addressbook_id, card_id),
        }
    }

    pub fn update_card(&mut self, card: Card) -> Result<()> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.update_card(card),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.update_card(card),
        }
    }

    pub fn delete_card(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<()> {
        match self {
            Self::None => bail!("Missing addressbook backend"),
            #[cfg(feature = "carddav")]
            Self::Carddav(client) => client.delete_card(addressbook_id, card_id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.delete_card(addressbook_id, card_id),
        }
    }
}
