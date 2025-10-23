use std::{collections::HashSet, path::PathBuf};

use anyhow::{anyhow, Result};
use io_addressbook::{
    addressbook::Addressbook,
    card::Card,
    vdir::coroutines::{
        create_addressbook::{CreateAddressbook, CreateAddressbookResult},
        create_card::{CreateCard, CreateCardResult},
        delete_addressbook::{DeleteAddressbook, DeleteAddressbookResult},
        delete_card::{DeleteCard, DeleteCardResult},
        list_addressbooks::{ListAddressbooks, ListAddressbooksResult},
        list_cards::{ListCards, ListCardsResult},
        read_card::{ReadCard, ReadCardResult},
        update_addressbook::{UpdateAddressbook, UpdateAddressbookResult},
        update_card::{UpdateCard, UpdateCardResult},
    },
};
use io_fs::runtimes::std::handle;

use super::config::VdirConfig;

#[derive(Debug)]
pub struct VdirClient {
    home_dir: PathBuf,
}

impl VdirClient {
    pub fn new(config: &VdirConfig) -> Self {
        Self {
            home_dir: config.home_dir.to_owned(),
        }
    }

    pub fn create_addressbook(&mut self, addressbook: Addressbook) -> Result<()> {
        let mut create = CreateAddressbook::new(&self.home_dir, addressbook);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                CreateAddressbookResult::Ok => break Ok(()),
                CreateAddressbookResult::Err(err) => {
                    return Err(anyhow!(err).context("Creat addressbook error"))
                }
                CreateAddressbookResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn list_addressbooks(&mut self) -> Result<HashSet<Addressbook>> {
        let mut list = ListAddressbooks::new(&self.home_dir);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                ListAddressbooksResult::Ok(addressbooks) => break Ok(addressbooks),
                ListAddressbooksResult::Err(err) => {
                    return Err(anyhow!(err).context("List addressbooks error"))
                }
                ListAddressbooksResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn update_addressbook(&mut self, addressbook: Addressbook) -> Result<()> {
        let mut update = UpdateAddressbook::new(&self.home_dir, addressbook);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                UpdateAddressbookResult::Ok => break Ok(()),
                UpdateAddressbookResult::Err(err) => {
                    return Err(anyhow!(err).context("Update addressbook error"))
                }
                UpdateAddressbookResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn delete_addressbook(&mut self, id: impl AsRef<str>) -> Result<()> {
        let mut delete = DeleteAddressbook::new(&self.home_dir, id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                DeleteAddressbookResult::Ok => break Ok(()),
                DeleteAddressbookResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                DeleteAddressbookResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn create_card(&mut self, card: Card) -> Result<()> {
        let mut create = CreateCard::new(&self.home_dir, card);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                CreateCardResult::Ok => break Ok(()),
                CreateCardResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                CreateCardResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn list_cards(&mut self, addressbook_id: impl AsRef<str>) -> Result<HashSet<Card>> {
        let mut list = ListCards::new(&self.home_dir, addressbook_id);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                ListCardsResult::Ok(ok) => break Ok(ok),
                ListCardsResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                ListCardsResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn read_card(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<Card> {
        let mut read = ReadCard::new(&self.home_dir, addressbook_id, card_id);
        let mut arg = None;

        loop {
            match read.resume(arg.take()) {
                ReadCardResult::Ok(card) => break Ok(card),
                ReadCardResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                ReadCardResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn update_card(&mut self, card: Card) -> Result<()> {
        let mut update = UpdateCard::new(&self.home_dir, card);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                UpdateCardResult::Ok => break Ok(()),
                UpdateCardResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                UpdateCardResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn delete_card(
        &mut self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<()> {
        let mut delete = DeleteCard::new(&self.home_dir, addressbook_id, card_id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                DeleteCardResult::Ok => break Ok(()),
                DeleteCardResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete addressbook error"))
                }
                DeleteCardResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }
}
