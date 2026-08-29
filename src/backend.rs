//! # Backend selector
//!
//! The value of the global `--backend` flag, naming which backend a
//! shared command runs against.
//!
//! `Auto` takes the first configured backend in the command's own
//! priority order, a named value pins that one and bails when the
//! account declares no matching block. Protocol-specific commands
//! ignore the flag, since their backend is their name.

use std::{fmt, str::FromStr};

use anyhow::{Error, bail};
use clap::Parser;

/// The backend a shared command targets.
#[derive(Clone, Copy, Debug, Default, Parser, PartialEq, Eq)]
pub enum Backend {
    /// The first backend the account configures, in priority order.
    #[default]
    Auto,
    /// CardDAV, over io-webdav.
    #[cfg(feature = "carddav")]
    Carddav,
    /// JMAP for contacts, over io-jmap.
    #[cfg(feature = "jmap")]
    Jmap,
    /// Microsoft Graph, over io-msgraph.
    #[cfg(feature = "msgraph")]
    Msgraph,
    /// Google People, over io-people.
    #[cfg(feature = "people")]
    People,
    /// A local pimdir store, over io-pimdir.
    #[cfg(feature = "pimdir")]
    Pimdir,
    /// A local vdir folder, over io-vdir.
    #[cfg(feature = "vdir")]
    Vdir,
}

#[allow(unused)]
impl Backend {
    /// Whether the CardDAV arm of a shared command is allowed to run.
    #[cfg(feature = "carddav")]
    pub fn allows_carddav(self) -> bool {
        matches!(self, Self::Auto | Self::Carddav)
    }

    /// Whether the JMAP arm of a shared command is allowed to run.
    #[cfg(feature = "jmap")]
    pub fn allows_jmap(self) -> bool {
        matches!(self, Self::Auto | Self::Jmap)
    }

    /// Whether the Microsoft Graph arm of a shared command may run.
    #[cfg(feature = "msgraph")]
    pub fn allows_msgraph(self) -> bool {
        matches!(self, Self::Auto | Self::Msgraph)
    }

    /// Whether the Google People arm of a shared command may run.
    #[cfg(feature = "people")]
    pub fn allows_people(self) -> bool {
        matches!(self, Self::Auto | Self::People)
    }

    /// Whether the pimdir arm of a shared command is allowed to run.
    #[cfg(feature = "pimdir")]
    pub fn allows_pimdir(self) -> bool {
        matches!(self, Self::Auto | Self::Pimdir)
    }

    /// Whether the vdir arm of a shared command is allowed to run.
    #[cfg(feature = "vdir")]
    pub fn allows_vdir(self) -> bool {
        matches!(self, Self::Auto | Self::Vdir)
    }
}

impl FromStr for Backend {
    type Err = Error;

    fn from_str(backend: &str) -> Result<Self, Self::Err> {
        match backend {
            "auto" => Ok(Self::Auto),
            #[cfg(feature = "carddav")]
            "carddav" => Ok(Self::Carddav),
            #[cfg(feature = "jmap")]
            "jmap" => Ok(Self::Jmap),
            #[cfg(feature = "msgraph")]
            "msgraph" => Ok(Self::Msgraph),
            #[cfg(feature = "people")]
            "people" => Ok(Self::People),
            #[cfg(feature = "pimdir")]
            "pimdir" => Ok(Self::Pimdir),
            #[cfg(feature = "vdir")]
            "vdir" => Ok(Self::Vdir),
            backend => bail!("Invalid backend {backend}"),
        }
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            #[cfg(feature = "carddav")]
            Self::Carddav => write!(f, "carddav"),
            #[cfg(feature = "jmap")]
            Self::Jmap => write!(f, "jmap"),
            #[cfg(feature = "msgraph")]
            Self::Msgraph => write!(f, "msgraph"),
            #[cfg(feature = "people")]
            Self::People => write!(f, "people"),
            #[cfg(feature = "pimdir")]
            Self::Pimdir => write!(f, "pimdir"),
            #[cfg(feature = "vdir")]
            Self::Vdir => write!(f, "vdir"),
        }
    }
}
