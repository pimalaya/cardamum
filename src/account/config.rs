//! # Account configuration
//!
//! Module dedicated to account configuration.

use serde::{Deserialize, Serialize};

#[cfg(feature = "_carddav")]
use crate::carddav::config::CardDavConfig;

#[cfg(not(feature = "_carddav"))]
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
    #[cfg(feature = "_carddav")]
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
            #[cfg(feature = "_carddav")]
            BackendDeserializer::CardDav(config) => Ok(Self::CardDav(config)),
            #[cfg(not(feature = "_carddav"))]
            BackendDeserializer::CardDav(_) => {
                Err("missing cargo feature `carddav`, `carddav-native-tls` or `carddav-rustls`")
            }
        }
    }
}
