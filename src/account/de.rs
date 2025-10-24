// This file is part of Cardamum, a CLI to manage contacts.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

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
