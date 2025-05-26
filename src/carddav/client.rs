use std::{collections::HashSet, net::TcpStream};

use anyhow::{bail, Context, Result};
use io_addressbook::{
    carddav::{self, coroutines::ListAddressbooks},
    Addressbook,
};
#[cfg(feature = "keyring")]
use io_keyring::{coroutines::Read as ReadEntry, runtimes::std::handle as handle_keyring};
#[cfg(feature = "command")]
use io_process::{coroutines::SpawnThenWaitWithOutput, runtimes::std::handle as handle_process};
use io_stream::{runtimes::std::handle, Io};
use secrecy::SecretString;

use super::{
    config::{Auth, CarddavConfig, Ssl},
    Secret, Stream,
};

pub struct Client {
    config: CarddavConfig,
    stream: Stream,
}

impl Client {
    pub fn new(config: CarddavConfig) -> Result<Self> {
        let stream = Self::connect(&config)?;
        Ok(Self { config, stream })
    }

    fn connect(config: &CarddavConfig) -> Result<Stream> {
        match &config.ssl {
            Ssl::Plain => {
                let tcp = TcpStream::connect((config.host.as_str(), config.port))?;
                Ok(Stream::Plain(tcp))
            }
            Ssl::Rustls { crypto } => {
                use std::sync::Arc;

                use rustls::{ClientConfig, ClientConnection, StreamOwned};
                use rustls_platform_verifier::ConfigVerifierExt;

                use super::config::RustlsCrypto;

                let provider = match crypto {
                    #[cfg(feature = "rustls-aws-lc")]
                    RustlsCrypto::Aws => rustls::crypto::aws_lc_rs::default_provider(),
                    #[cfg(not(feature = "rustls-aws-lc"))]
                    RustlsCrypto::Aws => {
                        bail!("Missing `rustls-aws-lc` cargo feature");
                    }

                    #[cfg(feature = "rustls-ring")]
                    RustlsCrypto::Ring => rustls::crypto::ring::default_provider(),
                    #[cfg(not(feature = "rustls-ring"))]
                    RustlsCrypto::Ring => {
                        bail!("Missing `rustls-ring` cargo feature");
                    }
                };

                if let Err(_) = provider.install_default() {
                    bail!("Install Rustls crypto provider error");
                }

                let conf = ClientConfig::with_platform_verifier();
                let sname = config.host.clone().try_into().unwrap();
                let conn = ClientConnection::new(Arc::new(conf), sname).unwrap();
                let tcp = TcpStream::connect((config.host.as_str(), config.port)).unwrap();
                let tls = StreamOwned::new(conn, tcp);

                Ok(Stream::Rustls(tls))
            }
            #[cfg(feature = "native-tls")]
            Ssl::NativeTls => {
                use native_tls::TlsConnector;

                let connector = TlsConnector::new()?;
                let tcp = TcpStream::connect((config.host.as_str(), config.port))?;
                let tls = connector.connect(&config.host, tcp)?;

                Ok(Stream::NativeTls(tls))
            }
        }
    }

    pub fn basic_auth(&self) -> Result<Option<(&String, SecretString)>> {
        let Auth::Basic { username, password } = &self.config.auth else {
            return Ok(None);
        };

        let secret = match password {
            Secret::Plain(secret) => secret.clone(),
            #[cfg(feature = "command")]
            Secret::Command(cmd) => {
                let mut spawn = SpawnThenWaitWithOutput::new(cmd.clone());
                let mut arg = None;

                loop {
                    match spawn.resume(arg.take()) {
                        Ok(output) => {
                            if !output.status.success() {
                                let err = String::from_utf8_lossy(&output.stderr);
                                bail!("get basic auth password from command error: {err}");
                            }

                            let secret = String::from_utf8_lossy(&output.stdout);
                            let Some(secret) = secret.lines().next() else {
                                break SecretString::default();
                            };

                            break SecretString::from(secret);
                        }
                        Err(io) => {
                            arg = Some(handle_process(io).context("get basic auth error")?);
                        }
                    }
                }
            }
            #[cfg(feature = "keyring")]
            Secret::Keyring(entry) => {
                let mut read = ReadEntry::new(entry.clone());
                let mut arg = None;

                loop {
                    match read.resume(arg.take()) {
                        Ok(secret) => {
                            break secret;
                        }
                        Err(io) => {
                            arg = Some(handle_keyring(io).context("get basic auth error")?);
                        }
                    }
                }
            }
        };

        Ok(Some((username, secret)))
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
            home_uri: self.config.home_uri.clone(),
            http_version: Default::default(),
            authentication: match self.basic_auth()? {
                Some((user, pass)) => carddav::config::Authentication::Basic(user.to_owned(), pass),
                None => carddav::config::Authentication::None,
            },
        };

        let mut list = ListAddressbooks::new(&config);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                Ok(addressbooks) => break Ok(addressbooks),
                Err(Io::Error(err)) => bail!("list addressbooks error: {err}"),
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
