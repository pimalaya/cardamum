//! # Account configuration
//!
//! Module dedicated to account configuration.

use serde::Deserialize;

#[cfg(feature = "carddav")]
use crate::carddav::config::CarddavConfig;
#[cfg(feature = "vdir")]
use crate::vdir::config::VdirConfig;

use super::de;

/// The account configuration.
#[derive(Clone, Debug, Deserialize)]
#[serde(from = "de::Account")]
pub struct Account {
    pub default: bool,
    #[cfg(feature = "carddav")]
    pub carddav: Option<CarddavConfig>,
    #[cfg(feature = "vdir")]
    pub vdir: Option<VdirConfig>,
}
