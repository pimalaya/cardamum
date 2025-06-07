use pimalaya_toolbox::{secret::Secret, stream::Tls};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CarddavConfig {
    #[serde(default)]
    pub default: bool,

    pub host: String,
    pub port: u16,

    #[serde(default)]
    pub auth: Auth,
    #[serde(default)]
    pub tls: Tls,

    #[serde(default = "CarddavConfig::default_home")]
    pub home: String,
}

impl CarddavConfig {
    pub fn default_home() -> String {
        String::from("/")
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Auth {
    #[default]
    Plain,
    Bearer(Secret),
    Basic {
        username: String,
        password: Secret,
    },
}
