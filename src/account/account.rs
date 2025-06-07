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
#[serde(try_from = "de::Account")]
pub enum Account {
    None,
    #[cfg(feature = "carddav")]
    Carddav(CarddavConfig),
    #[cfg(feature = "vdir")]
    Vdir(VdirConfig),
}
