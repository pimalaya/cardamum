#[allow(unused)]
use anyhow::{bail, Error};
use serde::Deserialize;

#[cfg(feature = "carddav")]
use crate::carddav::config::CarddavConfig;
#[cfg(feature = "vdir")]
use crate::vdir::config::VdirConfig;
#[allow(unused)]
use pimalaya_toolbox::feat;

#[cfg(not(feature = "carddav"))]
pub type CarddavConfig = ();
#[cfg(not(feature = "vdir"))]
pub type VdirConfig = ();

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Account {
    #[cfg_attr(not(feature = "carddav"), serde(deserialize_with = "carddav"))]
    Carddav(CarddavConfig),
    #[cfg_attr(not(feature = "vdir"), serde(deserialize_with = "vdir"))]
    Vdir(VdirConfig),
}

impl TryFrom<Account> for super::Account {
    type Error = Error;

    fn try_from(account: Account) -> Result<Self, Self::Error> {
        match account {
            #[cfg(feature = "carddav")]
            Account::Carddav(cmd) => Ok(Self::Carddav(cmd)),
            #[cfg(not(feature = "carddav"))]
            Account::Carddav(_) => bail!(feat!("carddav")),
            #[cfg(feature = "vdir")]
            Account::Vdir(entry) => Ok(Self::Vdir(entry)),
            #[cfg(not(feature = "vdir"))]
            Account::Vdir(_) => bail!(feat!("vdir")),
        }
    }
}

#[cfg(not(feature = "carddav"))]
pub fn carddav<'de, T, D: serde::Deserializer<'de>>(_: D) -> Result<T, D::Error> {
    Err(serde::de::Error::custom(feat!("carddav")))
}

#[cfg(not(feature = "vdir"))]
pub fn vdir<'de, T, D: serde::Deserializer<'de>>(_: D) -> Result<T, D::Error> {
    Err(serde::de::Error::custom(feat!("vdir")))
}
