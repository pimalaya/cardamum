//! # Account configuration
//!
//! Module dedicated to account configuration.

use serde::{Deserialize, Serialize};

#[cfg(any(
    feature = "carddav",
    feature = "carddav-native-tls",
    feature = "carddav-rustls",
))]
use crate::carddav::config::CardDavConfig;

#[cfg(not(feature = "carddav"))]
#[cfg(not(feature = "carddav-native-tls"))]
#[cfg(not(feature = "carddav-rustls"))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CardDavConfig {}

/// The account configuration.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TomlAccountConfig {
    /// The defaultness of the current account.
    #[serde(default)]
    pub default: bool,
    pub backend: Backend,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", content = "conf")]
#[serde(try_from = "BackendDeserializer")]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    #[serde(skip_serializing)]
    None,
    #[cfg(any(
        feature = "carddav",
        feature = "carddav-native-tls",
        feature = "carddav-rustls",
    ))]
    CardDav(CardDavConfig),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(tag = "type", content = "conf")]
#[serde(rename_all = "lowercase")]
pub enum BackendDeserializer {
    CardDav(CardDavConfig),
}

impl TryFrom<BackendDeserializer> for Backend {
    type Error = &'static str;

    fn try_from(backend: BackendDeserializer) -> Result<Self, Self::Error> {
        match backend {
            #[cfg(any(
                feature = "carddav",
                feature = "carddav-native-tls",
                feature = "carddav-rustls",
            ))]
            BackendDeserializer::CardDav(config) => Ok(Self::CardDav(config)),
            #[cfg(not(feature = "carddav"))]
            #[cfg(not(feature = "carddav-native-tls"))]
            #[cfg(not(feature = "carddav-rustls"))]
            BackendDeserializer::CardDav(_) => {
                Err("missing cargo feature `carddav`, `carddav-native-tls` or `carddav-rustls`")
            }
        }
    }
}
