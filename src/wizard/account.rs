//! Shared account configuration flow.
//!
//! Both the first-run wizard and `account configure` walk the exact
//! same prompts (backend picker, then the chosen backend's setup); the
//! only difference is where the defaults come from. There is a single
//! flow: CardDAV always runs discovery (existing accounts seed the auth
//! defaults and skip the email prompt). Editing therefore re-resolves
//! the server, so a moved account fixes itself on the next configure.

#[cfg(any(feature = "vdir", feature = "carddav"))]
use core::fmt;
use std::path::Path;

use anyhow::Result;
#[cfg(not(any(feature = "vdir", feature = "carddav")))]
use anyhow::bail;
#[cfg(any(feature = "vdir", feature = "carddav"))]
use pimalaya_cli::prompt;
use pimalaya_cli::spinner::Spinner;
#[cfg(feature = "carddav")]
use pimalaya_config::toml::TomlConfig;

#[cfg(feature = "vdir")]
use crate::config::VdirConfig;
use crate::config::{AccountConfig, Config};
#[cfg(feature = "carddav")]
use crate::config::{CarddavAuthConfig, CarddavConfig};
#[cfg(feature = "carddav")]
use crate::wizard::carddav;

/// Backend offered by the wizard. The config allows several backends
/// per account for naming convenience; the wizard proposes one.
#[cfg(any(feature = "vdir", feature = "carddav"))]
#[derive(Clone, Copy, Eq, PartialEq)]
enum Backend {
    #[cfg(feature = "carddav")]
    Carddav,
    #[cfg(feature = "vdir")]
    Vdir,
}

#[cfg(any(feature = "vdir", feature = "carddav"))]
impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "carddav")]
            Self::Carddav => f.write_str("Remote CardDAV"),
            #[cfg(feature = "vdir")]
            Self::Vdir => f.write_str("Local vdir"),
        }
    }
}

/// Builds an account through the backend picker and the chosen
/// backend's prompts. `existing` supplies the defaults when editing;
/// `None` starts a fresh account.
#[cfg(any(feature = "vdir", feature = "carddav"))]
#[cfg_attr(not(feature = "carddav"), allow(unused_variables))]
pub fn configure(
    account_name: &str,
    default: bool,
    existing: Option<AccountConfig>,
) -> Result<AccountConfig> {
    let backends = vec![
        #[cfg(feature = "carddav")]
        Backend::Carddav,
        #[cfg(feature = "vdir")]
        Backend::Vdir,
    ];

    let backend = prompt::item("Backend:", backends, default_backend(existing.as_ref()))?;

    #[cfg(feature = "vdir")]
    let vdir = match backend {
        Backend::Vdir => {
            let default_home = existing
                .as_ref()
                .and_then(|a| a.vdir.as_ref())
                .map(|v| v.home_dir.clone());
            let home_dir = prompt::text("Vdir home directory:", default_home.as_deref())?;
            Some(VdirConfig { home_dir })
        }
        #[cfg(feature = "carddav")]
        Backend::Carddav => None,
    };

    #[cfg(feature = "carddav")]
    let carddav = match backend {
        Backend::Carddav => {
            let existing = existing.as_ref().and_then(|a| a.carddav.as_ref());
            Some(configure_carddav(account_name, existing)?)
        }
        #[cfg(feature = "vdir")]
        Backend::Vdir => None,
    };

    Ok(AccountConfig {
        default,
        table: existing
            .as_ref()
            .map(|a| a.table.clone())
            .unwrap_or_default(),
        addressbook: existing
            .as_ref()
            .map(|a| a.addressbook.clone())
            .unwrap_or_default(),
        card: existing
            .as_ref()
            .map(|a| a.card.clone())
            .unwrap_or_default(),
        #[cfg(feature = "vdir")]
        vdir,
        #[cfg(feature = "carddav")]
        carddav,
    })
}

#[cfg(not(any(feature = "vdir", feature = "carddav")))]
pub fn configure(
    _account_name: &str,
    _default: bool,
    _existing: Option<AccountConfig>,
) -> Result<AccountConfig> {
    bail!("At least one backend (vdir or carddav) must be enabled at compile time")
}

/// Writes `config` to `target`, reporting progress with a spinner.
/// Shared by both wizards so they end the same way.
pub fn write(config: &Config, target: &Path) -> Result<()> {
    let at_path = format!("at {}", target.display());
    let spinner = Spinner::start(format!("Writing config {at_path}"));

    match config.write(target) {
        Ok(()) => {
            spinner.success(format!("Config written {at_path}"));
            Ok(())
        }
        Err(err) => {
            spinner.failure(format!("Cannot write config {at_path}"));
            Err(err)
        }
    }
}

/// Picks the backend an existing account already uses, so the picker
/// lands on it by default when editing.
#[cfg(any(feature = "vdir", feature = "carddav"))]
fn default_backend(existing: Option<&AccountConfig>) -> Option<Backend> {
    let existing = existing?;

    #[cfg(feature = "carddav")]
    if existing.carddav.is_some() {
        return Some(Backend::Carddav);
    }

    #[cfg(feature = "vdir")]
    if existing.vdir.is_some() {
        return Some(Backend::Vdir);
    }

    None
}

#[cfg(feature = "carddav")]
fn configure_carddav(
    account_name: &str,
    existing: Option<&CarddavConfig>,
) -> Result<CarddavConfig> {
    // The email seeds discovery. When editing, recover it from the
    // existing account instead of re-prompting; only a new account asks.
    let email = match existing.and_then(carddav_email) {
        Some(email) => email,
        None => prompt::text("Email address:", None)?,
    };

    carddav::configure(Config::project_name(), account_name, &email, existing)
}

/// Recovers the account's email from its existing config: a Basic
/// username that looks like an address, or the address Google embeds in
/// the home-set path (`.../principals/<email>/...`).
#[cfg(feature = "carddav")]
fn carddav_email(existing: &CarddavConfig) -> Option<String> {
    if let CarddavAuthConfig::Basic { username, .. } = &existing.auth
        && username.contains('@')
    {
        return Some(username.clone());
    }

    let mut segments = existing.home.as_ref()?.path_segments()?;
    segments.find(|segment| *segment == "principals")?;
    let email = segments.next()?;

    email.contains('@').then(|| email.to_owned())
}
