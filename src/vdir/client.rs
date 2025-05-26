use std::collections::HashSet;

use anyhow::{bail, Result};
use io_addressbook::{vdir::coroutines::ListAddressbooks, Addressbook};
use io_fs::{runtimes::std::handle, Io};

use super::config::VdirConfig;

pub struct Client {
    config: VdirConfig,
}

impl Client {
    pub fn new(config: VdirConfig) -> Self {
        Self { config }
    }

    // pub fn create_addressbook(&self, addressbook: Addressbook) -> Result<()> {
    //     let mut create = CreateAddressbook::new(addressbook);
    //     let mut arg = None;

    //     self.execute(&mut flow)?;
    //     flow.output().ok_or(eyre!("cannot get created addressbook"))
    // }

    pub fn list_addressbooks(&self) -> Result<HashSet<Addressbook>> {
        let mut list = ListAddressbooks::new(&self.config.home_dir);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                Ok(addressbooks) => break Ok(addressbooks),
                Err(Io::Error(err)) => bail!("list addressbooks error: {err}"),
                Err(io) => arg = Some(handle(io).context("list addressbooks error")),
            }
        }
    }

    // pub fn update_addressbook(
    //     &self,
    //     addressbook: PartialAddressbook,
    // ) -> Result<PartialAddressbook> {
    //     let mut flow = self.client.update_addressbook(addressbook);
    //     self.execute(&mut flow)?;
    //     flow.output().ok_or(eyre!("cannot get updated addressbook"))
    // }

    // pub fn delete_addressbook(&self, id: impl AsRef<str>) -> Result<bool> {
    //     let mut flow = self.client.delete_addressbook(id);
    //     self.execute(&mut flow)?;
    //     Ok(flow.output().is_some())
    // }

    // pub fn create_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
    //     let mut flow = self.client.create_card(addressbook_id, card);
    //     self.execute(&mut flow)?;
    //     flow.output().ok_or(eyre!("cannot get created card"))
    // }

    // pub fn read_card(
    //     &self,
    //     addressbook_id: impl AsRef<str>,
    //     card_id: impl ToString,
    // ) -> Result<Card> {
    //     let mut flow = self.client.read_card(addressbook_id, card_id);
    //     self.execute(&mut flow)?;
    //     flow.output().ok_or(eyre!("cannot get read card"))
    // }

    // pub fn list_cards(&self, addressbook_id: impl AsRef<str>) -> Result<Cards> {
    //     let mut flow = self.client.list_cards(addressbook_id);
    //     self.execute(&mut flow)?;
    //     flow.output().ok_or(eyre!("cannot get cards"))
    // }

    // pub fn update_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
    //     let mut flow = self.client.update_card(addressbook_id, card);
    //     self.execute(&mut flow)?;
    //     flow.output().ok_or(eyre!("cannot get updated card"))
    // }

    // pub fn delete_card(
    //     &self,
    //     addressbook_id: impl AsRef<str>,
    //     card_id: impl AsRef<str>,
    // ) -> Result<()> {
    //     let mut flow = self.client.delete_card(addressbook_id, card_id);
    //     self.execute(&mut flow)?;
    //     Ok(())
    // }
}
