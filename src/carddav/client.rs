use addressbook::{
    tcp::{self, Flow},
    Addressbook, Addressbooks, Card, Cards, PartialAddressbook,
};
use color_eyre::Result;

use super::config::{CardDavConfig, Encryption};

pub struct Client {
    config: CardDavConfig,
    client: addressbook::carddav::Client,
}

impl Client {
    pub fn new(config: CardDavConfig) -> Result<Self> {
        let client = addressbook::carddav::Client::try_from(config.clone())?;
        Ok(Self { config, client })
    }

    pub fn create_addressbook(&self, addressbook: Addressbook) -> Result<Addressbook> {
        let mut flow = self.client.create_addressbook(addressbook);
        self.execute(&mut flow)?;
        Ok(flow.output()?)
    }

    pub fn list_addressbooks(&self) -> Result<Addressbooks> {
        let mut flow = self.client.list_addressbooks();
        self.execute(&mut flow)?;
        Ok(flow.output()?)
    }

    pub fn list_cards(&self, addressbook_id: impl AsRef<str>) -> Result<Cards> {
        let mut flow = self.client.list_cards(addressbook_id);
        self.execute(&mut flow)?;
        Ok(flow.output()?)
    }

    pub fn update_addressbook(
        &self,
        addressbook: PartialAddressbook,
    ) -> Result<PartialAddressbook> {
        let mut flow = self.client.update_addressbook(addressbook);
        self.execute(&mut flow)?;
        Ok(flow.output()?)
    }

    pub fn delete_addressbook(&self, id: impl AsRef<str>) -> Result<bool> {
        let mut flow = self.client.delete_addressbook(id);
        self.execute(&mut flow)?;
        Ok(flow.output()?)
    }

    pub fn create_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
        let mut flow = self.client.create_card(addressbook_id, card);
        self.execute(&mut flow)?;
        Ok(flow.output())
    }

    pub fn read_card(
        &self,
        addressbook_id: impl AsRef<str>,
        card_id: impl ToString,
    ) -> Result<Card> {
        let mut flow = self.client.read_card(addressbook_id, card_id);
        self.execute(&mut flow)?;
        Ok(flow.output()?)
    }

    pub fn update_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
        let mut flow = self.client.update_card(addressbook_id, card);
        self.execute(&mut flow)?;
        Ok(flow.output())
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
        F: Flow<Item = tcp::Io>,
        F: tcp::Read + tcp::Write,
    {
        match &self.config.encryption {
            #[cfg(feature = "carddav")]
            Encryption::None => {
                use addressbook_carddav::Connector;

                let mut tcp = Connector::connect(&self.config.hostname, self.config.port)?;

                while let Some(io) = flow.next() {
                    match io {
                        tcp::Io::Read => {
                            tcp.read(flow)?;
                        }
                        tcp::Io::Write => {
                            tcp.write(flow)?;
                        }
                    }
                }
            }
            #[cfg(feature = "carddav-native-tls")]
            Encryption::NativeTls(_) => {
                use addressbook_carddav_native_tls::Connector;

                let mut tls = Connector::connect(&self.config.hostname, self.config.port)?;

                while let Some(io) = flow.next() {
                    match io {
                        tcp::Io::Read => {
                            tls.read(flow)?;
                        }
                        tcp::Io::Write => {
                            tls.write(flow)?;
                        }
                    }
                }
            }
            #[cfg(feature = "_carddav-rustls")]
            Encryption::Rustls(config) => {
                use addressbook_carddav_rustls::{Connector, CryptoProvider};

                use crate::carddav::config::RustlsCrypto;

                let crypto = match config.crypto {
                    RustlsCrypto::Default => CryptoProvider::Default,
                    #[cfg(feature = "carddav-rustls-aws-lc")]
                    RustlsCrypto::AwsLc => CryptoProvider::AwsLc,
                    #[cfg(feature = "carddav-rustls-ring")]
                    RustlsCrypto::Ring => CryptoProvider::Ring,
                };

                let mut tls = Connector::connect(&self.config.hostname, self.config.port, &crypto)?;

                while let Some(io) = flow.next() {
                    match io {
                        tcp::Io::Read => {
                            tls.read(flow)?;
                        }
                        tcp::Io::Write => {
                            tls.write(flow)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
