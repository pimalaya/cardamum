//! Shared account configuration flow.
//!
//! Both the first-run wizard and `account configure` start from one
//! endpoint prompt, mirroring the cardamum-android onboarding screen: a
//! single field takes an email address, a server URL, or a local vdir
//! path, and its shape orients the rest of the setup. An email (or bare
//! domain) feeds pimconf's parallel search (fixed provider rules, PACC,
//! RFC 6764 DAV resolve, RFC 8620 JMAP resolve), and every discovered
//! service and authentication method becomes one selectable entry (see
//! [`crate::wizard::search`]). A `scheme://` URL is a CardDAV server to
//! configure by hand. A filesystem path is a local vdir; unlike Android,
//! it is validated for existence. Editing an existing account seeds the
//! endpoint prompt with its current value.

use std::path::Path;

use anyhow::{Result, bail};
use pimalaya_cli::{prompt, spinner::Spinner};
#[cfg(feature = "carddav")]
use pimalaya_config::toml::TomlConfig;
#[cfg(feature = "carddav")]
use url::Url;

#[cfg(feature = "carddav")]
use crate::carddav::client::parse_carddav_server;
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

/// The endpoint prompt label, shared by the create and edit flows.
const ENDPOINT_PROMPT: &str = "Email address, server URL, or vdir path:";

/// What the endpoint input resolved to, which orients the backend setup.
enum Target {
    #[cfg(feature = "vdir")]
    Vdir(String),
    #[cfg(feature = "carddav")]
    Server(Url),
    Discover(String),
}

/// The backend config produced by the chosen flow, folded into the
/// [`AccountConfig`] afterwards.
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

/// Creates a fresh account: endpoint first, then the account name, then
/// the oriented backend setup. Returns the chosen name alongside its
/// config so the caller can key it in the file.
pub fn configure_new() -> Result<(String, AccountConfig)> {
    let target = prompt_endpoint(None)?;
    let account_name = prompt::text("Account name:", Some("personal"))?;
    let config = configure_target(&account_name, true, None, target)?;

    Ok((account_name, config))
}

/// Re-configures the already-named `account_name`: the endpoint prompt is
/// seeded from `existing`, then the oriented backend setup runs with its
/// current values as defaults.
pub fn configure_existing(
    account_name: &str,
    default: bool,
    existing: Option<AccountConfig>,
) -> Result<AccountConfig> {
    let hint = existing.as_ref().and_then(endpoint_hint);
    let target = prompt_endpoint(hint.as_deref())?;

    configure_target(account_name, default, existing, target)
}

/// Prompts the endpoint until it resolves to a valid [`Target`],
/// re-asking on a malformed input or a vdir path that does not exist.
fn prompt_endpoint(default: Option<&str>) -> Result<Target> {
    loop {
        let input = prompt::text(ENDPOINT_PROMPT, default)?;

        match classify(input.trim()) {
            #[cfg(feature = "vdir")]
            Ok(Target::Vdir(path)) => {
                let expanded = shellexpand::tilde(&path);
                if Path::new(expanded.as_ref()).is_dir() {
                    return Ok(Target::Vdir(path));
                }
                eprintln!("No such vdir directory `{path}`");
            }
            Ok(target) => return Ok(target),
            Err(err) => eprintln!("{err}"),
        }
    }
}

/// Reads the shape of the endpoint input: a filesystem path is a local
/// vdir, a `scheme://` URL is a CardDAV server, anything else is an
/// email (or bare domain) to feed discovery.
fn classify(input: &str) -> Result<Target> {
    if input.is_empty() {
        bail!("Empty endpoint; enter an email address, a server URL, or a vdir path");
    }

    if is_path(input) {
        #[cfg(feature = "vdir")]
        return Ok(Target::Vdir(vdir_path(input)));
        #[cfg(not(feature = "vdir"))]
        bail!("`{input}` looks like a vdir path, but vdir support is not compiled in");
    }

    if input.contains("://") {
        #[cfg(feature = "carddav")]
        return Ok(Target::Server(parse_carddav_server(input)?));
        #[cfg(not(feature = "carddav"))]
        bail!("`{input}` looks like a server URL, but CardDAV support is not compiled in");
    }

    Ok(Target::Discover(input.to_owned()))
}

/// Whether the input names a filesystem path (absolute, home-relative,
/// explicitly relative, or a `file://` URL) rather than a network
/// endpoint.
fn is_path(input: &str) -> bool {
    input.starts_with("file://")
        || input.starts_with('/')
        || input.starts_with('~')
        || input.starts_with("./")
        || input.starts_with("../")
}

/// Strips the `file://` scheme from a path input, leaving a bare path.
#[cfg(feature = "vdir")]
fn vdir_path(input: &str) -> String {
    input.strip_prefix("file://").unwrap_or(input).to_owned()
}

/// Orients the backend setup from the resolved endpoint, then folds the
/// chosen backend into a fresh [`AccountConfig`] (reusing `existing`'s
/// rendering options when editing).
#[cfg_attr(not(feature = "carddav"), allow(unused_variables))]
fn configure_target(
    account_name: &str,
    default: bool,
    existing: Option<AccountConfig>,
    target: Target,
) -> Result<AccountConfig> {
    let chosen = match target {
        #[cfg(feature = "vdir")]
        Target::Vdir(home_dir) => Chosen::Vdir(VdirConfig { home_dir }),
        #[cfg(feature = "carddav")]
        Target::Server(url) => {
            let existing = existing.as_ref().and_then(|a| a.carddav.as_ref());
            Chosen::Carddav(carddav::configure_server(
                Config::project_name(),
                account_name,
                &url,
                existing,
            )?)
        }
        Target::Discover(email) => configure_discovery(&email)?,
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
fn configure_discovery(email: &str) -> Result<Chosen> {
    let spinner = Spinner::start("Searching services");
    let mut found = search::search(email)?;
    retain_supported(&mut found);
    spinner.success(format!("Found {} service(s)", found.len()));

    if found.is_empty() {
        bail!("No contacts service discovered for `{email}`; enter a server URL instead");
    }

    let choice = prompt::item("Service:", found, None)?;

    match &choice.kind {
        #[cfg(feature = "carddav")]
        DiscoveredKind::Carddav(url) => Ok(Chosen::Carddav(carddav::configure_discovered(
            email, url, &choice,
        )?)),
        #[cfg(feature = "jmap")]
        DiscoveredKind::Jmap(_) => Ok(Chosen::Jmap(jmap::configure(email, None, Some(&choice))?)),
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

/// Best-effort endpoint to seed the edit prompt from an existing
/// account: the vdir path, the CardDAV email or URL, or the JMAP server.
fn endpoint_hint(existing: &AccountConfig) -> Option<String> {
    #[cfg(feature = "vdir")]
    if let Some(vdir) = &existing.vdir {
        return Some(vdir.home_dir.clone());
    }

    #[cfg(feature = "carddav")]
    if let Some(carddav) = &existing.carddav {
        // Prefer the email, which re-runs discovery; otherwise a URL the
        // server route can reuse verbatim.
        return carddav_email(carddav)
            .or_else(|| carddav.server.clone())
            .or_else(|| carddav.home.as_ref().map(Url::to_string))
            .or_else(|| carddav.discover.clone());
    }

    #[cfg(feature = "jmap")]
    if let Some(jmap) = &existing.jmap {
        return Some(jmap.server.clone());
    }

    None
}

/// Recovers the account's email from its existing CardDAV config: a
/// Basic username that looks like an address, or the address Google
/// embeds in the home-set path (`.../principals/<email>/...`).
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
