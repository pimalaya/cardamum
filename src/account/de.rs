#[allow(unused)]
use pimalaya_toolbox::feat;
use serde::Deserialize;

#[cfg(feature = "carddav")]
use crate::carddav::config::CarddavConfig;
#[cfg(feature = "vdir")]
use crate::vdir::config::VdirConfig;

#[cfg(not(feature = "carddav"))]
pub type CarddavConfig = ();
#[cfg(not(feature = "vdir"))]
pub type VdirConfig = ();

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Account {
    #[serde(default)]
    pub default: bool,
    #[cfg_attr(not(feature = "carddav"), serde(default, deserialize_with = "carddav"))]
    pub carddav: Option<CarddavConfig>,
    #[cfg_attr(not(feature = "vdir"), serde(default, deserialize_with = "vdir"))]
    pub vdir: Option<VdirConfig>,
}

impl From<Account> for super::Account {
    fn from(account: Account) -> Self {
        super::Account {
            default: account.default,
            #[cfg(feature = "carddav")]
            carddav: account.carddav,
            #[cfg(feature = "vdir")]
            vdir: account.vdir,
        }
    }
}

// pub fn uri<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Url, D::Error> {
//     let uri = Url::deserialize(deserializer)?;

//     let scheme = uri.scheme();
//     let carddav = scheme.starts_with("http");
//     let vdir = scheme == "file" || !uri.has_authority();

//     #[cfg(not(feature = "carddav"))]
//     if carddav {
//         return Err(Error::custom(feat!("carddav")));
//     }

//     #[cfg(not(feature = "vdir"))]
//     if vdir {
//         return Err(Error::custom(feat!("vdir")));
//     }

//     if !carddav && !vdir {
//         let expected = "`file`, `http`, `https`";
//         let err = format!("unknown scheme `{scheme}`, expected one of {expected}");
//         return Err(Error::custom(err));
//     }

//     Ok(uri)
// }

#[cfg(not(feature = "carddav"))]
pub fn carddav<'de, T, D: serde::Deserializer<'de>>(_: D) -> Result<T, D::Error> {
    Err(serde::de::Error::custom(feat!("carddav")))
}

#[cfg(not(feature = "vdir"))]
pub fn vdir<'de, T, D: serde::Deserializer<'de>>(_: D) -> Result<T, D::Error> {
    Err(serde::de::Error::custom(feat!("vdir")))
}
