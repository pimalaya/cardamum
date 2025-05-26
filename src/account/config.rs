//! # Account configuration
//!
//! Module dedicated to account configuration.

use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(feature = "carddav")]
use crate::carddav::config::CarddavConfig;
#[cfg(feature = "vdir")]
use crate::vdir::config::VdirConfig;

/// The account configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TomlAccountConfig {
    /// The defaultness of the current account.
    #[serde(default)]
    pub default: bool,
    pub backend: Backend,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    #[cfg(feature = "carddav")]
    Carddav(CarddavConfig),
    #[cfg(not(feature = "carddav"))]
    Carddav(MissingCarddavFeature),

    #[cfg(feature = "vdir")]
    Vdir(VdirConfig),
    #[cfg(not(feature = "vdir"))]
    Vdir(MissingVdirFeature),
}

trait MissingCargoFeature {
    const FEATURE_NAME: &'static str;

    fn message(&self) -> String {
        format!("missing `{}` cargo feature", Self::FEATURE_NAME)
    }
}

#[derive(Clone, Debug)]
pub struct MissingVdirFeature;

impl MissingCargoFeature for MissingVdirFeature {
    const FEATURE_NAME: &'static str = "vdir";
}

impl fmt::Display for MissingVdirFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Serialize for MissingVdirFeature {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(self.message()))
    }
}

impl<'de> serde::Deserialize<'de> for MissingVdirFeature {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom(Self.message()))
    }
}

pub struct MissingCarddavFeature;

impl MissingCargoFeature for MissingCarddavFeature {
    const FEATURE_NAME: &'static str = "carddav";
}

impl fmt::Display for MissingCarddavFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Serialize for MissingCarddavFeature {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(self.message()))
    }
}

impl<'de> serde::Deserialize<'de> for MissingCarddavFeature {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom(Self.message()))
    }
}
