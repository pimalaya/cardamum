//! Shared account configuration flow.
//!
//! Both the first-run wizard and `account configure` walk the exact
//! same prompts: a backend picker (email discovery first, then one
//! entry per backend), then the chosen backend's setup. Discovery
//! mirrors the cardamum-android configuration screen: the email feeds
//! pimconf's parallel search and every discovered service and
//! authentication method combination becomes one selectable entry
//! (see [`crate::wizard::search`]). Editing an existing account seeds
//! the defaults of its backend flow.

use core::fmt;
use std::path::Path;

use anyhow::{Result, bail};
use pimalaya_cli::{prompt, spinner::Spinner};
#[cfg(feature = "carddav")]
use pimalaya_config::toml::TomlConfig;

#[cfg(feature = "vdir")]
use crate::config::VdirConfig;
use crate::config::{AccountConfig, Config};
#[cfg(feature = "carddav")]
use crate::config::{CarddavAuthConfig, CarddavConfig};
#[cfg(feature = "carddav")]
use crate::wizard::carddav;
#[cfg(feature = "google")]
use crate::wizard::google;
#[cfg(feature = "jmap")]
use crate::wizard::jmap;
#[cfg(feature = "msgraph")]
use crate::wizard::msgraph;
use crate::wizard::search::{self, DiscoveredKind};

/// Backend offered by the wizard. The config allows several backends
/// per account for naming convenience; the wizard proposes one.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Backend {
    Discover,
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

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discover => f.write_str("Automatic discovery (from email address)"),
            #[cfg(feature = "carddav")]
            Self::Carddav => f.write_str("Remote CardDAV"),
            #[cfg(feature = "jmap")]
            Self::Jmap => f.write_str("Remote JMAP"),
            #[cfg(feature = "msgraph")]
            Self::Msgraph => f.write_str("Microsoft Graph API"),
            #[cfg(feature = "google")]
            Self::Google => f.write_str("Google People API"),
            #[cfg(feature = "vdir")]
            Self::Vdir => f.write_str("Local vdir"),
        }
    }
}

/// The backend config produced by the chosen wizard flow, folded into
/// the [`AccountConfig`] afterwards.
enum Chosen {
    #[cfg(feature = "vdir")]
    Vdir(VdirConfig),
    #[cfg(feature = "carddav")]
    Carddav(CarddavConfig),
    #[cfg(feature = "jmap")]
    Jmap(crate::config::JmapConfig),
    #[cfg(feature = "msgraph")]
    Msgraph(crate::config::MsgraphConfig),
    #[cfg(feature = "google")]
    Google(crate::config::GoogleConfig),
}

/// Builds an account through the backend picker and the chosen
/// backend's prompts. `existing` supplies the defaults when editing;
/// `None` starts a fresh account.
#[cfg_attr(not(feature = "carddav"), allow(unused_variables))]
pub fn configure(
    account_name: &str,
    default: bool,
    existing: Option<AccountConfig>,
) -> Result<AccountConfig> {
    let backends = vec![
        Backend::Discover,
        #[cfg(feature = "carddav")]
        Backend::Carddav,
        #[cfg(feature = "jmap")]
        Backend::Jmap,
        #[cfg(feature = "msgraph")]
        Backend::Msgraph,
        #[cfg(feature = "google")]
        Backend::Google,
        #[cfg(feature = "vdir")]
        Backend::Vdir,
    ];

    let backend = prompt::item("Backend:", backends, default_backend(existing.as_ref()))?;

    let chosen = match backend {
        Backend::Discover => configure_discovery()?,
        #[cfg(feature = "carddav")]
        Backend::Carddav => {
            let existing = existing.as_ref().and_then(|a| a.carddav.as_ref());
            Chosen::Carddav(configure_carddav(account_name, existing)?)
        }
        #[cfg(feature = "jmap")]
        Backend::Jmap => {
            let existing = existing.as_ref().and_then(|a| a.jmap.as_ref());
            let email = prompt::text("Email address:", None)?;
            Chosen::Jmap(jmap::configure(&email, existing, None)?)
        }
        #[cfg(feature = "msgraph")]
        Backend::Msgraph => {
            let existing = existing.as_ref().and_then(|a| a.msgraph.as_ref());
            Chosen::Msgraph(msgraph::configure(existing)?)
        }
        #[cfg(feature = "google")]
        Backend::Google => {
            let existing = existing.as_ref().and_then(|a| a.google.as_ref());
            Chosen::Google(google::configure(existing)?)
        }
        #[cfg(feature = "vdir")]
        Backend::Vdir => {
            let default_home = existing
                .as_ref()
                .and_then(|a| a.vdir.as_ref())
                .map(|v| v.home_dir.clone());
            let home_dir = prompt::text("Vdir home directory:", default_home.as_deref())?;
            Chosen::Vdir(VdirConfig { home_dir })
        }
    };

    let mut config = AccountConfig {
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
        ..Default::default()
    };

    match chosen {
        #[cfg(feature = "vdir")]
        Chosen::Vdir(vdir) => config.vdir = Some(vdir),
        #[cfg(feature = "carddav")]
        Chosen::Carddav(carddav) => config.carddav = Some(carddav),
        #[cfg(feature = "jmap")]
        Chosen::Jmap(jmap) => config.jmap = Some(jmap),
        #[cfg(feature = "msgraph")]
        Chosen::Msgraph(msgraph) => config.msgraph = Some(msgraph),
        #[cfg(feature = "google")]
        Chosen::Google(google) => config.google = Some(google),
    }

    Ok(config)
}

/// Runs the email-driven discovery flow: search the services reachable
/// from the address, let the user pick one, and configure its backend.
fn configure_discovery() -> Result<Chosen> {
    let email = prompt::text("Email address:", None)?;

    let spinner = Spinner::start("Searching services");
    let mut found = search::search(&email)?;
    retain_supported(&mut found);
    spinner.success(format!("Found {} service(s)", found.len()));

    if found.is_empty() {
        bail!("No contacts service discovered for `{email}`; configure one manually");
    }

    let choice = prompt::item("Service:", found, None)?;

    match &choice.kind {
        #[cfg(feature = "carddav")]
        DiscoveredKind::Carddav(url) => Ok(Chosen::Carddav(carddav::configure_discovered(
            &email, url, &choice,
        )?)),
        #[cfg(feature = "jmap")]
        DiscoveredKind::Jmap(_) => Ok(Chosen::Jmap(jmap::configure(&email, None, Some(&choice))?)),
        #[cfg(feature = "msgraph")]
        DiscoveredKind::Msgraph => Ok(Chosen::Msgraph(msgraph::configure(None)?)),
        #[cfg(feature = "google")]
        DiscoveredKind::Google => Ok(Chosen::Google(google::configure(None)?)),
        #[allow(unreachable_patterns)]
        kind => bail!("Discovered service `{kind:?}` is not compiled in"),
    }
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

/// Drops the discovered entries whose backend is not compiled in.
fn retain_supported(found: &mut Vec<search::Discovered>) {
    found.retain(|entry| match entry.kind {
        DiscoveredKind::Carddav(_) => cfg!(feature = "carddav"),
        DiscoveredKind::Jmap(_) => cfg!(feature = "jmap"),
        DiscoveredKind::Msgraph => cfg!(feature = "msgraph"),
        DiscoveredKind::Google => cfg!(feature = "google"),
    });
}

/// Picks the backend an existing account already uses, so the picker
/// lands on it by default when editing.
fn default_backend(existing: Option<&AccountConfig>) -> Option<Backend> {
    let existing = existing?;

    #[cfg(feature = "carddav")]
    if existing.carddav.is_some() {
        return Some(Backend::Carddav);
    }

    #[cfg(feature = "jmap")]
    if existing.jmap.is_some() {
        return Some(Backend::Jmap);
    }

    #[cfg(feature = "msgraph")]
    if existing.msgraph.is_some() {
        return Some(Backend::Msgraph);
    }

    #[cfg(feature = "google")]
    if existing.google.is_some() {
        return Some(Backend::Google);
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
