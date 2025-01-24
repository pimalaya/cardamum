use addressbook::{vdir::fs, Addressbook, Addressbooks, Card, Cards, PartialAddressbook};
use addressbook_vdir::Connector;
use color_eyre::{eyre::eyre, Result};

use super::config::VdirConfig;

pub struct Client {
    client: addressbook::vdir::Client,
    vdir: Connector,
}

impl Client {
    pub fn new(config: VdirConfig) -> Self {
        let client = addressbook::vdir::Client::from(config);
        let vdir = Connector::new();
        Self { client, vdir }
    }

    pub fn create_addressbook(&self, addressbook: Addressbook) -> Result<Addressbook> {
        let mut flow = self.client.create_addressbook(addressbook);
        self.execute(&mut flow)?;
        flow.output().ok_or(eyre!("cannot get created addressbook"))
    }

    pub fn list_addressbooks(&self) -> Result<Addressbooks> {
        let mut flow = self.client.list_addressbooks();
        self.execute(&mut flow)?;
        flow.output().ok_or(eyre!("cannot get listed addressbooks"))
    }

    pub fn update_addressbook(
        &self,
        addressbook: PartialAddressbook,
    ) -> Result<PartialAddressbook> {
        let mut flow = self.client.update_addressbook(addressbook);
        self.execute(&mut flow)?;
        flow.output().ok_or(eyre!("cannot get updated addressbook"))
    }

    pub fn delete_addressbook(&self, id: impl AsRef<str>) -> Result<bool> {
        let mut flow = self.client.delete_addressbook(id);
        self.execute(&mut flow)?;
        Ok(flow.output().is_some())
    }

    pub fn create_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
        let mut flow = self.client.create_card(addressbook_id, card);
        self.execute(&mut flow)?;
        flow.output().ok_or(eyre!("cannot get created card"))
    }

    pub fn read_card(
        &self,
        addressbook_id: impl AsRef<str>,
        card_id: impl ToString,
    ) -> Result<Card> {
        let mut flow = self.client.read_card(addressbook_id, card_id);
        self.execute(&mut flow)?;
        flow.output().ok_or(eyre!("cannot get read card"))
    }

    pub fn list_cards(&self, addressbook_id: impl AsRef<str>) -> Result<Cards> {
        let mut flow = self.client.list_cards(addressbook_id);
        self.execute(&mut flow)?;
        flow.output().ok_or(eyre!("cannot get cards"))
    }

    pub fn update_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
        let mut flow = self.client.update_card(addressbook_id, card);
        self.execute(&mut flow)?;
        flow.output().ok_or(eyre!("cannot get updated card"))
    }

    pub fn delete_card(
        &self,
        addressbook_id: impl AsRef<str>,
        card_id: impl AsRef<str>,
    ) -> Result<()> {
        let mut flow = self.client.delete_card(addressbook_id, card_id);
        self.execute(&mut flow)?;
        Ok(())
    }

    fn execute<F>(&self, flow: &mut F) -> Result<()>
    where
        F: Iterator<Item = fs::Io>,
        F: AsMut<fs::State>,
    {
        while let Some(io) = flow.next() {
            self.vdir.execute(flow, io)?;
        }

        Ok(())
    }
}
