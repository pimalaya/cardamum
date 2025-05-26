use serde::{Deserialize, Serialize};

use super::Secret;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CarddavConfig {
    #[serde(default)]
    pub default: bool,

    pub host: String,
    pub port: u16,

    pub auth: Auth,
    #[serde(default)]
    pub ssl: Ssl,

    #[serde(default = "CarddavConfig::default_home")]
    pub home: String,
}

impl CarddavConfig {
    pub fn default_home() -> String {
        String::from("/")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Auth {
    Plain,
    Basic { username: String, password: Secret },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ssl {
    Plain,
    NativeTls,
    Rustls { crypto: RustlsCrypto },
}

impl Default for Ssl {
    fn default() -> Self {
        Self::Rustls {
            crypto: Default::default(),
        }
    }
}

#[cfg(feature = "rustls")]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustlsCrypto {
    #[default]
    Aws,
    Ring,
}
