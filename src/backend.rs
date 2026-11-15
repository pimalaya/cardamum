//! # Backend selector
//!
//! The value of the global `--backend` flag, naming which backend a
//! unified command runs against.
//!
//! `Auto` takes the first configured backend in the command's own
//! priority order, a named value pins that one and bails when the
//! account declares no matching block. Protocol-specific commands
//! ignore the flag, since their backend is their name.
//!
//! A backend the build leaves out still names its configuration block,
//! which loads unread so one file serves every build: a command reaching
//! it fails coded `backend-not-built`, naming the cargo feature.

use std::{fmt, str::FromStr};

use anyhow::{Error, bail};
use clap::Parser;
use pimalaya_cli::error::CodedError;

use crate::error::ErrorCode;

/// Every backend an account may configure, in the order `auto` resolves
/// them. Each one's cargo feature is its name.
pub const NAMES: [&str; 6] = ["vdir", "pimdir", "carddav", "jmap", "msgraph", "gpeople"];

/// Whether this build carries the backend `name`.
pub fn is_built(name: &str) -> bool {
    let built = [
        cfg!(feature = "vdir"),
        cfg!(feature = "pimdir"),
        cfg!(feature = "carddav"),
        cfg!(feature = "jmap"),
        cfg!(feature = "msgraph"),
        cfg!(feature = "gpeople"),
    ];

    let mut names = NAMES.into_iter().zip(built);
    names.any(|(known, built)| built && known == name)
}

/// The failure of a command reaching the backend `name`, which this build
/// leaves out.
pub fn not_built(name: &str) -> Error {
    let message =
        format!("This build leaves out the `{name}` backend, rebuild with `--features {name}`");
    CodedError::new(ErrorCode::BackendNotBuilt, message).into()
}

/// The backend a unified command targets.
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
    /// Google People, over io-gpeople.
    #[cfg(feature = "gpeople")]
    Gpeople,
    /// A local pimdir store, over io-pimdir.
    #[cfg(feature = "pimdir")]
    Pimdir,
    /// A local vdir folder, over io-vdir.
    #[cfg(feature = "vdir")]
    Vdir,
}

#[allow(unused)]
impl Backend {
    /// Whether the CardDAV arm of a unified command is allowed to run.
    #[cfg(feature = "carddav")]
    pub fn allows_carddav(self) -> bool {
        matches!(self, Self::Auto | Self::Carddav)
    }

    /// Whether the JMAP arm of a unified command is allowed to run.
    #[cfg(feature = "jmap")]
    pub fn allows_jmap(self) -> bool {
        matches!(self, Self::Auto | Self::Jmap)
    }

    /// Whether the Microsoft Graph arm of a unified command may run.
    #[cfg(feature = "msgraph")]
    pub fn allows_msgraph(self) -> bool {
        matches!(self, Self::Auto | Self::Msgraph)
    }

    /// Whether the Google People arm of a unified command may run.
    #[cfg(feature = "gpeople")]
    pub fn allows_gpeople(self) -> bool {
        matches!(self, Self::Auto | Self::Gpeople)
    }

    /// Whether the pimdir arm of a unified command is allowed to run.
    #[cfg(feature = "pimdir")]
    pub fn allows_pimdir(self) -> bool {
        matches!(self, Self::Auto | Self::Pimdir)
    }

    /// Whether the vdir arm of a unified command is allowed to run.
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
            #[cfg(feature = "gpeople")]
            "gpeople" => Ok(Self::Gpeople),
            #[cfg(feature = "pimdir")]
            "pimdir" => Ok(Self::Pimdir),
            #[cfg(feature = "vdir")]
            "vdir" => Ok(Self::Vdir),
            backend if NAMES.contains(&backend) => Err(not_built(backend)),
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
            #[cfg(feature = "gpeople")]
            Self::Gpeople => write!(f, "gpeople"),
            #[cfg(feature = "pimdir")]
            Self::Pimdir => write!(f, "pimdir"),
            #[cfg(feature = "vdir")]
            Self::Vdir => write!(f, "vdir"),
        }
    }
}
