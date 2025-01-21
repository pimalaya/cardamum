//! # Account configuration
//!
//! Module dedicated to account configuration.

use std::process::Command;

use addressbook::{
    carddav::{self, Client},
    tcp::{self, Flow},
};
use color_eyre::{eyre::Error, Result};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// The account configuration.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TomlAccountConfig {
    /// The defaultness of the current account.
    #[serde(default)]
    pub default: bool,
    pub backend: Backend,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Backend {
    CardDav(CardDavConfig),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CardDavConfig {
    #[serde(alias = "host")]
    pub hostname: String,
    pub port: u16,
    #[serde(alias = "auth")]
    pub authentication: Authentication,
    pub encryption: Encryption,

    pub home_uri: Option<String>,
    pub http_version: Option<HttpVersion>,
}

impl CardDavConfig {
    const DEFAULT_ADDRESSBOOK_HOME_SET_URI: &'static str = "/";
    const DEFAULT_HTTP_VERSION: &'static str = "1.1";

    pub fn home_uri(&self) -> &str {
        self.home_uri
            .as_deref()
            .unwrap_or(Self::DEFAULT_ADDRESSBOOK_HOME_SET_URI)
    }

    pub fn http_version(&self) -> &str {
        match self.http_version.as_ref() {
            Some(v) => v.as_ref(),
            None => Self::DEFAULT_HTTP_VERSION,
        }
    }
}

impl TryFrom<CardDavConfig> for Client {
    type Error = Error;

    fn try_from(config: CardDavConfig) -> Result<Client> {
        let mut client = Client::new();

        client.config.home_uri = config.home_uri().to_owned();
        client.config.http_version = match config.http_version {
            Some(HttpVersion::V1_0) => carddav::client::HttpVersion::V1_0,
            Some(HttpVersion::V1_1) => carddav::client::HttpVersion::V1_1,
            None => carddav::client::HttpVersion::default(),
        };

        if let Authentication::Basic(auth) = config.authentication {
            let mut args = auth.password_command.split_whitespace();
            let program = args.next().unwrap();
            let mut password_bytes = Command::new(program).args(args).output()?.stdout;
            let password = String::from_utf8(password_bytes.trim_ascii().to_owned())?;
            client.set_basic_authentication(auth.username, password);
            password_bytes.zeroize();
        }

        client.config.hostname = config.hostname;
        client.config.port = config.port;

        Ok(client)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum HttpVersion {
    #[serde(rename = "1.0")]
    V1_0,
    #[default]
    #[serde(rename = "1.1")]
    V1_1,
}

impl AsRef<str> for HttpVersion {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Authentication {
    None,
    Basic(BasicAuthentication),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct BasicAuthentication {
    pub username: String,
    #[serde(alias = "password-cmd")]
    pub password_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Encryption {
    None,
    NativeTls(#[serde(default)] NativeTls),
    Rustls(#[serde(default)] Rustls),
}

impl Encryption {
    pub fn run<F>(&self, client: &Client, flow: &mut F) -> Result<()>
    where
        F: Flow<Item = tcp::Io>,
        F: tcp::Read + tcp::Write,
    {
        match self {
            Encryption::None => {
                let mut tcp = addressbook_std::Connector::connect(&client.config)?;

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
            Encryption::NativeTls(_) => {
                let mut tls = addressbook_std_native_tls::Connector::connect(&client.config)?;

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
            Encryption::Rustls(_) => {
                let mut tls = addressbook_std_rustls::Connector::connect(&client.config)?;

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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct NativeTls;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Rustls {
    #[serde(default)]
    crypto_provider: Option<RustlsCryptoProvider>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustlsCryptoProvider {
    AwsLc,
    Ring,
}

// impl Into<(AccountConfig, Backend)> for TomlAccountConfig {
//     fn into(self) -> (AccountConfig, Backend) {
//         let TomlAccountConfig { default, backend } = self;
//         let config = AccountConfig { default };
//         (config, backend)
//     }
// }
