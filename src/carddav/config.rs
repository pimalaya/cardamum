use std::process::Command;

use addressbook::carddav::{self, Client};
use color_eyre::{eyre::Error, Result};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

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
#[serde(tag = "type", content = "conf")]
#[serde(rename_all = "kebab-case")]
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
#[serde(tag = "type", content = "conf")]
#[serde(try_from = "EncryptionDeserializer")]
#[serde(rename_all = "kebab-case")]
pub enum Encryption {
    #[cfg(feature = "carddav")]
    None,
    #[cfg(feature = "carddav-native-tls")]
    NativeTls(#[serde(default)] NativeTls),
    #[cfg(feature = "_carddav-rustls")]
    Rustls(#[serde(default)] Rustls),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "conf")]
#[serde(rename_all = "kebab-case")]
pub enum EncryptionDeserializer {
    None,
    NativeTls(NativeTls),
    Rustls(Rustls),
}

impl TryFrom<EncryptionDeserializer> for Encryption {
    type Error = &'static str;

    fn try_from(encryption: EncryptionDeserializer) -> Result<Self, Self::Error> {
        match encryption {
            #[cfg(feature = "carddav")]
            EncryptionDeserializer::None => Ok(Self::None),
            #[cfg(not(feature = "carddav"))]
            EncryptionDeserializer::None => Err("missing cargo feature `carddav`"),
            #[cfg(feature = "carddav-native-tls")]
            EncryptionDeserializer::NativeTls(config) => Ok(Self::NativeTls(config)),
            #[cfg(not(feature = "carddav-native-tls"))]
            EncryptionDeserializer::NativeTls(_) => {
                Err("missing cargo feature `carddav-native-tls`")
            }
            #[cfg(feature = "_carddav-rustls")]
            EncryptionDeserializer::Rustls(config) => Ok(Self::Rustls(config)),
            #[cfg(not(feature = "carddav-rustls-aws-lc"))]
            #[cfg(not(feature = "carddav-rustls-ring"))]
            EncryptionDeserializer::Rustls(_) => Err("missing cargo feature `carddav-rustls`"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NativeTls {}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "_carddav-rustls", serde(deny_unknown_fields))]
pub struct Rustls {
    #[cfg(feature = "_carddav-rustls")]
    #[serde(default)]
    pub crypto: RustlsCrypto,
}

#[cfg(feature = "_carddav-rustls")]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RustlsCryptoDeserializer")]
#[serde(rename_all = "kebab-case")]
pub enum RustlsCrypto {
    #[default]
    Default,
    #[cfg(feature = "carddav-rustls-aws-lc")]
    AwsLc,
    #[cfg(feature = "carddav-rustls-ring")]
    Ring,
}

#[cfg(feature = "_carddav-rustls")]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustlsCryptoDeserializer {
    #[default]
    Default,
    AwsLc,
    Ring,
}

#[cfg(feature = "_carddav-rustls")]
impl TryFrom<RustlsCryptoDeserializer> for RustlsCrypto {
    type Error = &'static str;

    fn try_from(crypto: RustlsCryptoDeserializer) -> Result<Self, Self::Error> {
        match crypto {
            #[cfg(feature = "carddav-rustls-aws-lc")]
            RustlsCryptoDeserializer::AwsLc => Ok(Self::AwsLc),
            #[cfg(not(feature = "carddav-rustls-aws-lc"))]
            RustlsCryptoDeserializer::AwsLc => Err("missing cargo feature `carddav-rustls-aws-lc`"),
            #[cfg(feature = "carddav-rustls-ring")]
            RustlsCryptoDeserializer::Ring => Ok(Self::Ring),
            #[cfg(not(feature = "carddav-rustls-ring"))]
            RustlsCryptoDeserializer::Ring => Err("missing cargo feature `carddav-rustls-ring`"),
            RustlsCryptoDeserializer::Default => Ok(Self::Default),
        }
    }
}
