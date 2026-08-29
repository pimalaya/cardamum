//! # Account discovery
//!
//! The half of the wizard that decides what the account is: one prompt
//! takes an email address, a server URL or a local folder path, and its
//! shape orients the setup.
//!
//! An email, a bare domain or a server URL runs io-pim-discovery's
//! parallel discovery (see [`super::search`]) and every reachable service
//! becomes one selectable configuration, whose authentication method is
//! prompted next. A folder is a local vdir or pimdir store.
//!
//! What becomes of the discovered account, a file to create, a block to
//! append or a document on stdout, belongs to [`super::configure`], which
//! is also where the welcome and the prompts around this one live.
//!
//! Cardamum runs no OAuth 2.0 grant itself: a grant only unlocks the
//! external token brokers (Ortie, pizauth, oama) behind the API token
//! credential prompt (see [`super::secret`]).

use std::path::Path;

use anyhow::{Context, Result, bail};
use pimalaya_cli::{prompt, spinner::Spinner};
use url::Url;

#[cfg(feature = "carddav")]
use crate::config::CarddavConfig;
#[cfg(feature = "jmap")]
use crate::config::JmapConfig;
#[cfg(feature = "msgraph")]
use crate::config::MsgraphConfig;
#[cfg(feature = "people")]
use crate::config::PeopleConfig;
#[cfg(feature = "pimdir")]
use crate::config::PimdirConfig;
#[cfg(feature = "vdir")]
use crate::config::VdirConfig;
#[cfg(feature = "carddav")]
use crate::wizard::carddav;
#[cfg(feature = "jmap")]
use crate::wizard::jmap;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
use crate::wizard::local;
#[cfg(feature = "msgraph")]
use crate::wizard::msgraph;
#[cfg(feature = "people")]
use crate::wizard::people;
use crate::{
    account::check,
    config::AccountConfig,
    wizard::search::{self, Discovered, DiscoveredKind},
};

/// The label of the single wizard prompt.
const ENDPOINT_PROMPT: &str = "Email:";

/// The documented sample configuration, shown in the welcome banner and
/// pointed at when discovery finds nothing to configure automatically.
pub const CONFIG_SAMPLE_URL: &str =
    "https://github.com/pimalaya/cardamum/blob/master/config.sample.toml";

/// The backend config produced by the chosen flow, folded into a fresh
/// [`AccountConfig`] afterwards.
enum Chosen {
    #[cfg(feature = "vdir")]
    Vdir(VdirConfig),
    #[cfg(feature = "pimdir")]
    Pimdir(PimdirConfig),
    #[cfg(feature = "carddav")]
    Carddav(Box<CarddavConfig>),
    #[cfg(feature = "jmap")]
    Jmap(Box<JmapConfig>),
    #[cfg(feature = "msgraph")]
    Msgraph(MsgraphConfig),
    #[cfg(feature = "people")]
    People(PeopleConfig),
}

/// Discovers one account from a single prompt, tests it, and hands back
/// the name it proposes with the account itself.
pub fn run() -> Result<(String, AccountConfig)> {
    let input = prompt::text::<&str>(ENDPOINT_PROMPT, None)?;
    let input = input.trim();
    if input.is_empty() {
        bail!("Empty input: enter an email address, a server URL, or a folder path");
    }

    let account_name = default_account_name(input);
    let (account, tested) = build_account(&account_name, input)?;

    // NOTE: testing here stops a bad credential or endpoint from yielding
    // an account that cannot connect.
    if !tested {
        let spinner = Spinner::start("Testing account configuration");
        if let Err(err) = check::test_account(&account) {
            spinner.failure("Account configuration test failed");
            return Err(err);
        }
        spinner.success("Account configuration is valid");
    }

    Ok((account_name, account))
}

/// The chosen backend, and whether the flow already validated its
/// connection so the caller can skip the final account test.
struct Outcome {
    chosen: Chosen,
    tested: bool,
}

impl Outcome {
    /// A not-yet-tested outcome, for the flows deferring validation to the
    /// final account test (every backend today).
    fn untested(chosen: Chosen) -> Self {
        Self {
            chosen,
            tested: false,
        }
    }
}

/// Orients the setup from the input shape, then folds the chosen backend
/// into a fresh [`AccountConfig`].
///
/// The returned flag tells whether the flow already validated its
/// connection. The account is left non-default here: claiming the default
/// depends on the configuration it lands in, [`super::configure`]'s
/// business.
fn build_account(account_name: &str, input: &str) -> Result<(AccountConfig, bool)> {
    let Outcome { chosen, tested } = if is_path(input) {
        Outcome::untested(configure_local(input)?)
    } else {
        configure_discovery(account_name, input)?
    };

    let mut account = AccountConfig {
        default: false,
        ..Default::default()
    };

    match chosen {
        #[cfg(feature = "vdir")]
        Chosen::Vdir(vdir) => account.vdir = Some(vdir),
        #[cfg(feature = "pimdir")]
        Chosen::Pimdir(pimdir) => account.pimdir = Some(pimdir),
        #[cfg(feature = "carddav")]
        Chosen::Carddav(carddav) => account.carddav = Some(*carddav),
        #[cfg(feature = "jmap")]
        Chosen::Jmap(jmap) => account.jmap = Some(*jmap),
        #[cfg(feature = "msgraph")]
        Chosen::Msgraph(msgraph) => account.msgraph = Some(msgraph),
        #[cfg(feature = "people")]
        Chosen::People(people) => account.people = Some(people),
    }

    Ok((account, tested))
}

/// Runs the discovery flow for an email, a bare domain or a `scheme://`
/// server URL, then configures the picked service.
///
/// Only the services supported by this build, and matching the URL scheme
/// when one was given, are offered. Discovering nothing stops the wizard
/// (see [`stop_undiscovered`]).
fn configure_discovery(account_name: &str, input: &str) -> Result<Outcome> {
    let (email, scheme) = if input.contains("://") {
        let url = Url::parse(input).with_context(|| format!("Invalid server URL `{input}`"))?;
        let host = url.host_str().unwrap_or_default().to_string();
        (format!("@{host}"), Some(url.scheme().to_string()))
    } else if input.contains('@') {
        (input.to_string(), None)
    } else {
        (format!("@{input}"), None)
    };

    let spinner = Spinner::start("Searching for contacts services");
    let mut found = search::search(&email)?;
    retain_supported(&mut found);
    if let Some(scheme) = &scheme {
        retain_scheme(&mut found, scheme)?;
    }

    if found.is_empty() {
        spinner.failure("No configuration found");
        return stop_undiscovered(input);
    }
    spinner.success(format!("Found {} configuration(s)", found.len()));

    let default = found.first().cloned();
    let choice = prompt::item("Choose a configuration:", found, default)?;

    dispatch(account_name, &email, choice)
}

/// Keeps only the discovered entries a `scheme://` URL asked for.
///
/// The HTTP family and `carddav(s)` keep CardDAV, `jmap(s)` keeps JMAP. A
/// proprietary entry is dropped since the user named an open protocol, and
/// an unknown scheme is rejected outright.
fn retain_scheme(found: &mut Vec<Discovered>, scheme: &str) -> Result<()> {
    match scheme {
        "carddav" | "carddavs" | "http" | "https" => {
            found.retain(|entry| matches!(entry.kind, DiscoveredKind::Carddav(_)));
        }
        "jmap" | "jmaps" => {
            found.retain(|entry| matches!(entry.kind, DiscoveredKind::Jmap(_)));
        }
        other => bail!("Unsupported server scheme `{other}`"),
    }

    Ok(())
}

/// Stops the wizard when discovery found nothing to configure for `input`.
///
/// It points at the documented sample and errors out rather than dropping
/// into a hand-entry flow: the wizard only ever configures what it can
/// discover automatically.
fn stop_undiscovered(input: &str) -> Result<Outcome> {
    bail!(
        "Could not automatically discover a configuration for `{input}`.\n\n\
         Write your account configuration by hand instead, starting from the \
         documented sample:\n  {CONFIG_SAMPLE_URL}"
    )
}

/// Configures the backend behind a discovered entry.
///
/// None of them validates its connection inline, so every outcome defers
/// to the final account test.
#[cfg_attr(
    all(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "people"
    ),
    allow(unreachable_patterns)
)]
#[cfg_attr(
    not(any(feature = "carddav", feature = "jmap")),
    allow(unused_variables)
)]
fn dispatch(account_name: &str, email: &str, choice: Discovered) -> Result<Outcome> {
    match &choice.kind {
        #[cfg(feature = "carddav")]
        DiscoveredKind::Carddav(url) => Ok(Outcome::untested(Chosen::Carddav(Box::new(
            carddav::configure_discovered(account_name, email, url, &choice)?,
        )))),
        #[cfg(feature = "jmap")]
        DiscoveredKind::Jmap(_) => Ok(Outcome::untested(Chosen::Jmap(Box::new(
            jmap::configure_discovered(account_name, email, &choice)?,
        )))),
        #[cfg(feature = "msgraph")]
        DiscoveredKind::Msgraph => Ok(Outcome::untested(Chosen::Msgraph(msgraph::configure(
            account_name,
        )?))),
        #[cfg(feature = "people")]
        DiscoveredKind::People => Ok(Outcome::untested(Chosen::People(people::configure(
            account_name,
        )?))),
        kind => bail!("Configuration `{kind:?}` is not supported by this build"),
    }
}

/// Configures a local backend from a typed folder path.
#[cfg(any(feature = "vdir", feature = "pimdir"))]
fn configure_local(input: &str) -> Result<Chosen> {
    let raw = input.strip_prefix("file://").unwrap_or(input);
    let root = shellexpand::tilde(raw).into_owned();
    if !Path::new(&root).is_dir() {
        bail!("No such folder `{raw}`");
    }

    Ok(match local::configure(root.into())? {
        #[cfg(feature = "vdir")]
        local::Local::Vdir(config) => Chosen::Vdir(config),
        #[cfg(feature = "pimdir")]
        local::Local::Pimdir(config) => Chosen::Pimdir(config),
    })
}

/// Rejects a typed folder path, no local backend being compiled in.
#[cfg(not(any(feature = "vdir", feature = "pimdir")))]
fn configure_local(input: &str) -> Result<Chosen> {
    bail!("`{input}` looks like a folder path, but no local backend is compiled in")
}

/// Drops the discovered entries whose backend is not compiled in.
fn retain_supported(found: &mut Vec<Discovered>) {
    found.retain(|entry| match entry.kind {
        DiscoveredKind::Carddav(_) => cfg!(feature = "carddav"),
        DiscoveredKind::Jmap(_) => cfg!(feature = "jmap"),
        DiscoveredKind::Msgraph => cfg!(feature = "msgraph"),
        DiscoveredKind::People => cfg!(feature = "people"),
    });
}

/// Proposes a default account name from the input shape: the first label
/// of the domain, or the folder name of a local path.
fn default_account_name(input: &str) -> String {
    if is_path(input) {
        let raw = input.strip_prefix("file://").unwrap_or(input);
        return Path::new(raw)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("personal")
            .to_string();
    }

    if let Ok(url) = Url::parse(input)
        && let Some(host) = url.host_str()
    {
        return first_label(host);
    }

    match input.rsplit_once('@') {
        Some((_, domain)) => first_label(domain),
        None => first_label(input),
    }
}

/// The first dot-separated label of a host or domain.
fn first_label(host: &str) -> String {
    host.split('.').next().unwrap_or(host).to_string()
}

/// Whether the input names a filesystem path (absolute, home-relative,
/// explicitly relative, or a `file://` URL) rather than an endpoint.
fn is_path(input: &str) -> bool {
    input.starts_with("file://")
        || input.starts_with('/')
        || input.starts_with('~')
        || input.starts_with("./")
        || input.starts_with("../")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_name_defaults_to_the_first_domain_label() {
        assert_eq!(default_account_name("clement.douin@posteo.net"), "posteo");
        assert_eq!(default_account_name("alice@mail.example.co.uk"), "mail");
        assert_eq!(default_account_name("@posteo.net"), "posteo");
        assert_eq!(default_account_name("posteo.net"), "posteo");
    }

    #[test]
    fn account_name_defaults_to_the_last_path_component() {
        assert_eq!(
            default_account_name("/home/alice/contacts/personal"),
            "personal"
        );
        assert_eq!(default_account_name("~/contacts/work"), "work");
        assert_eq!(
            default_account_name("file:///var/contacts/archive"),
            "archive"
        );
    }

    #[test]
    fn an_unknown_scheme_is_rejected() {
        assert!(retain_scheme(&mut Vec::new(), "imap").is_err());
    }
}
