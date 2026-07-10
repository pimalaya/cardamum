use std::{fmt, str::FromStr};

use anyhow::{Error, bail};
use clap::Parser;

/// Selects which backend a cross-protocol command should target.
///
/// `Auto` lets the command pick the first configured-and-supported
/// backend in its own priority order. The named variants pin the
/// command to that backend; the command bails if it cannot be served
/// (config missing, or the operation has no arm for that backend).
///
/// The protocol-specific subcommands (vdir, carddav) ignore this arg
/// entirely.
#[derive(Clone, Copy, Debug, Default, Parser, PartialEq, Eq)]
pub enum Backend {
    #[default]
    Auto,
    #[cfg(feature = "carddav")]
    Carddav,
    #[cfg(feature = "jmap")]
    Jmap,
    #[cfg(feature = "msgraph")]
    Msgraph,
    #[cfg(feature = "google")]
    Google,
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

    /// Whether the Microsoft Graph arm of a shared command is allowed
    /// to run.
    #[cfg(feature = "msgraph")]
    pub fn allows_msgraph(self) -> bool {
        matches!(self, Self::Auto | Self::Msgraph)
    }

    /// Whether the Google People arm of a shared command is allowed to
    /// run.
    #[cfg(feature = "google")]
    pub fn allows_google(self) -> bool {
        matches!(self, Self::Auto | Self::Google)
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
            #[cfg(feature = "google")]
            "google" => Ok(Self::Google),
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
            #[cfg(feature = "google")]
            Self::Google => write!(f, "google"),
            #[cfg(feature = "vdir")]
            Self::Vdir => write!(f, "vdir"),
        }
    }
}
