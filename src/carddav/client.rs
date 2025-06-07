use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use io_addressbook::{
    carddav::{self, coroutines::ListAddressbooks},
    Addressbook,
};
use io_stream::{runtimes::std::handle, Io};
use pimalaya_toolbox::stream::Stream;

use super::config::{Auth, CarddavConfig};

#[derive(Debug)]
pub struct Client {
    config: CarddavConfig,
    stream: Stream,
}

impl Client {
    pub fn new(config: CarddavConfig) -> Result<Self> {
        let stream = Stream::connect(&config.host, config.port, &config.tls)?;
        Ok(Self { config, stream })
    }

    // pub fn create_addressbook(&self, addressbook: Addressbook) -> Result<Addressbook> {
    //     let mut flow = self.client.create_addressbook(addressbook);
    //     self.execute(&mut flow)?;
    //     Ok(flow.output()?)
    // }

    pub fn list_addressbooks(&mut self) -> Result<HashSet<Addressbook>> {
        let config = carddav::Config {
            host: self.config.host.clone(),
            port: self.config.port,
            home_uri: self.config.home.clone(),
            http_version: Default::default(),
            auth: match &self.config.auth {
                Auth::Plain => carddav::config::Auth::Plain,
                Auth::Bearer(token) => carddav::config::Auth::Bearer {
                    token: token.get()?,
                },
                Auth::Basic { username, password } => carddav::config::Auth::Basic {
                    username: username.clone(),
                    password: password.get()?,
                },
            },
        };

        let mut list = ListAddressbooks::new(&config);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                Ok(addressbooks) => break Ok(addressbooks),
                Err(Io::Error(err)) => return Err(anyhow!(err).context("List addressbooks error")),
                Err(io) => {
                    arg = Some(handle(&mut self.stream, io).context("list addressbooks error")?)
                }
            }
        }
    }

    // pub fn list_cards(&self, addressbook_id: impl AsRef<str>) -> Result<Cards> {
    //     let mut flow = self.client.list_cards(addressbook_id);
    //     self.execute(&mut flow)?;
    //     Ok(flow.output()?)
    // }

    // pub fn update_addressbook(
    //     &self,
    //     addressbook: PartialAddressbook,
    // ) -> Result<PartialAddressbook> {
    //     let mut flow = self.client.update_addressbook(addressbook);
    //     self.execute(&mut flow)?;
    //     Ok(flow.output()?)
    // }

    // pub fn delete_addressbook(&self, id: impl AsRef<str>) -> Result<bool> {
    //     let mut flow = self.client.delete_addressbook(id);
    //     self.execute(&mut flow)?;
    //     Ok(flow.output()?)
    // }

    // pub fn create_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
    //     let mut flow = self.client.create_card(addressbook_id, card);
    //     self.execute(&mut flow)?;
    //     Ok(flow.output())
    // }

    // pub fn read_card(
    //     &self,
    //     addressbook_id: impl AsRef<str>,
    //     card_id: impl ToString,
    // ) -> Result<Card> {
    //     let mut flow = self.client.read_card(addressbook_id, card_id);
    //     self.execute(&mut flow)?;
    //     Ok(flow.output()?)
    // }

    // pub fn update_card(&self, addressbook_id: impl AsRef<str>, card: Card) -> Result<Card> {
    //     let mut flow = self.client.update_card(addressbook_id, card);
    //     self.execute(&mut flow)?;
    //     Ok(flow.output())
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

    // fn execute<F>(&self, flow: &mut F) -> Result<()>
    // where
    //     F: Flow<Item = tcp::Io>,
    //     F: tcp::Read + tcp::Write,
    // {
    //     match &self.config.encryption {
    //         #[cfg(feature = "carddav")]
    //         Encryption::None => {
    //             use addressbookcarddav::Connector;

    //             let mut tcp = Connector::connect(&self.config.hostname, self.config.port)?;

    //             while let Some(io) = flow.next() {
    //                 match io {
    //                     tcp::Io::Read => {
    //                         tcp.read(flow)?;
    //                     }
    //                     tcp::Io::Write => {
    //                         tcp.write(flow)?;
    //                     }
    //                 }
    //             }
    //         }
    //         #[cfg(feature = "carddav-native-tls")]
    //         Encryption::NativeTls(_) => {
    //             use addressbookcarddav_native_tls::Connector;

    //             let mut tls = Connector::connect(&self.config.hostname, self.config.port)?;

    //             while let Some(io) = flow.next() {
    //                 match io {
    //                     tcp::Io::Read => {
    //                         tls.read(flow)?;
    //                     }
    //                     tcp::Io::Write => {
    //                         tls.write(flow)?;
    //                     }
    //                 }
    //             }
    //         }
    //         #[cfg(feature = "carddav-rustls")]
    //         Encryption::Rustls(config) => {
    //             use addressbookcarddav_rustls::{Connector, CryptoProvider};

    //             use crate::carddav::config::RustlsCrypto;

    //             let crypto = match config.crypto {
    //                 RustlsCrypto::Default => CryptoProvider::Default,
    //                 #[cfg(feature = "carddav-rustls-aws-lc")]
    //                 RustlsCrypto::AwsLc => CryptoProvider::AwsLc,
    //                 #[cfg(feature = "carddav-rustls-ring")]
    //                 RustlsCrypto::Ring => CryptoProvider::Ring,
    //             };

    //             let mut tls = Connector::connect(&self.config.hostname, self.config.port, &crypto)?;

    //             while let Some(io) = flow.next() {
    //                 match io {
    //                     tcp::Io::Read => {
    //                         tls.read(flow)?;
    //                     }
    //                     tcp::Io::Write => {
    //                         tls.write(flow)?;
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }
}
