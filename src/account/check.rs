//! # Account check command
//!
//! Reaches every backend the account configures and reports, one by one,
//! whether it answered.

use std::{fmt, path::PathBuf};

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use pimalaya_config::toml::TomlConfig;
use serde::Serialize;

use crate::{
    backend::Backend,
    config::{AccountConfig, Config},
};

/// Validate the account configuration.
///
/// Picks the active account, from the global `--account` flag or from
/// the default, then reaches every backend `--backend` allows. Each
/// check walks the same handshake and authentication path the other
/// commands take, so a bad credential surfaces here.
///
/// JSON output: `{"account", "backends": [{"backend", "ok", "error"}]}`.
#[derive(Debug, Parser)]
pub struct AccountCheckCommand;

impl AccountCheckCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        let mut config = match Config::from_paths_or_default(config_paths)? {
            Some(config) => config,
            None => bail!(
                "No configuration found at {}, run `cardamum configure` to generate one",
                Config::target_path(config_paths)?.display(),
            ),
        };

        let (name, account_config) = config
            .take_account(account_name)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No default account found, name one with `-a <NAME>` or mark one with `default = true`"
                )
            })?;

        let mut report = CheckReport {
            account: name,
            backends: Vec::new(),
        };

        #[cfg(feature = "vdir")]
        if backend.allows_vdir()
            && let Some(vdir_config) = &account_config.vdir
        {
            report
                .backends
                .push(BackendCheck::from("vdir", connect_vdir(vdir_config)));
        }

        #[cfg(feature = "pimdir")]
        if backend.allows_pimdir()
            && let Some(pimdir_config) = &account_config.pimdir
        {
            report
                .backends
                .push(BackendCheck::from("pimdir", connect_pimdir(pimdir_config)));
        }

        #[cfg(feature = "carddav")]
        if backend.allows_carddav()
            && let Some(carddav_config) = &account_config.carddav
        {
            report.backends.push(BackendCheck::from(
                "carddav",
                connect_carddav(carddav_config),
            ));
        }

        #[cfg(feature = "jmap")]
        if backend.allows_jmap()
            && let Some(jmap_config) = &account_config.jmap
        {
            report
                .backends
                .push(BackendCheck::from("jmap", connect_jmap(jmap_config)));
        }

        #[cfg(feature = "msgraph")]
        if backend.allows_msgraph()
            && let Some(msgraph_config) = &account_config.msgraph
        {
            report.backends.push(BackendCheck::from(
                "msgraph",
                connect_msgraph(msgraph_config),
            ));
        }

        #[cfg(feature = "people")]
        if backend.allows_people()
            && let Some(people_config) = &account_config.people
        {
            report
                .backends
                .push(BackendCheck::from("people", connect_people(people_config)));
        }

        if report.backends.is_empty() {
            bail!("No backend matching `{backend}` is configured for this account");
        }

        printer.out(report)
    }
}

/// Tests every backend the account configures, failing on the first error.
///
/// The wizard runs it over a freshly built account, so a bad credential
/// or endpoint stops the process instead of yielding a configuration
/// that cannot connect.
pub fn test_account(account_config: &AccountConfig) -> Result<()> {
    #[cfg(feature = "vdir")]
    if let Some(vdir_config) = &account_config.vdir {
        connect_vdir(vdir_config)?;
    }

    #[cfg(feature = "pimdir")]
    if let Some(pimdir_config) = &account_config.pimdir {
        connect_pimdir(pimdir_config)?;
    }

    #[cfg(feature = "carddav")]
    if let Some(carddav_config) = &account_config.carddav {
        connect_carddav(carddav_config)?;
    }

    #[cfg(feature = "jmap")]
    if let Some(jmap_config) = &account_config.jmap {
        connect_jmap(jmap_config)?;
    }

    #[cfg(feature = "msgraph")]
    if let Some(msgraph_config) = &account_config.msgraph {
        connect_msgraph(msgraph_config)?;
    }

    #[cfg(feature = "people")]
    if let Some(people_config) = &account_config.people {
        connect_people(people_config)?;
    }

    Ok(())
}

/// Checks the vdir home is a directory, a local tree having nothing to
/// connect to.
#[cfg(feature = "vdir")]
fn connect_vdir(vdir_config: &crate::config::VdirConfig) -> Result<()> {
    use std::path::Path;

    let home = Path::new(&vdir_config.home_dir);
    if !home.is_dir() {
        bail!(
            "Vdir home `{}` does not exist or is not a directory",
            home.display()
        );
    }

    Ok(())
}

/// Opens the pimdir store and reads its collection list.
///
/// That exercises the index the shared commands read through, a local
/// store having no handshake nor authentication to check.
#[cfg(feature = "pimdir")]
fn connect_pimdir(pimdir_config: &crate::config::PimdirConfig) -> Result<()> {
    use crate::pimdir::backend::PimdirBackend;

    PimdirBackend::new(pimdir_config.clone())?.list_addressbooks()?;
    Ok(())
}

/// Resolves the CardDAV context root, then walks the principal and the
/// addressbook home set, proving address, TLS and authentication work.
#[cfg(feature = "carddav")]
fn connect_carddav(carddav_config: &crate::config::CarddavConfig) -> Result<()> {
    use crate::carddav::client::open_carddav_client;

    open_carddav_client(carddav_config.clone())?;

    Ok(())
}

/// Establishes the JMAP session, proving address, TLS and authentication
/// work.
#[cfg(feature = "jmap")]
fn connect_jmap(jmap_config: &crate::config::JmapConfig) -> Result<()> {
    use crate::jmap::backend::JmapBackend;

    JmapBackend::new(jmap_config.clone())?;

    Ok(())
}

/// Lists the Graph contact folders, proving the token grants access to
/// the contacts API.
#[cfg(feature = "msgraph")]
fn connect_msgraph(msgraph_config: &crate::config::MsgraphConfig) -> Result<()> {
    use crate::msgraph::backend::MsgraphBackend;

    let mut client = MsgraphBackend::new(msgraph_config.clone())?;
    client.list_addressbooks()?;

    Ok(())
}

/// Lists the People contact groups, proving the token grants access to
/// the contacts API.
#[cfg(feature = "people")]
fn connect_people(people_config: &crate::config::PeopleConfig) -> Result<()> {
    use crate::people::backend::PeopleBackend;

    let mut client = PeopleBackend::new(people_config.clone())?;
    client.list_addressbooks()?;

    Ok(())
}

/// Result of a check: one outcome per configured backend.
#[derive(Clone, Debug, Serialize)]
pub struct CheckReport {
    pub account: String,
    pub backends: Vec<BackendCheck>,
}

/// Outcome of checking a single backend.
#[derive(Clone, Debug, Serialize)]
pub struct BackendCheck {
    pub backend: &'static str,
    pub ok: bool,
    pub error: Option<String>,
}

impl BackendCheck {
    fn from(backend: &'static str, result: Result<()>) -> Self {
        match result {
            Ok(()) => Self {
                backend,
                ok: true,
                error: None,
            },
            Err(err) => Self {
                backend,
                ok: false,
                error: Some(format!("{err:#}")),
            },
        }
    }
}

impl fmt::Display for CheckReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Account: {}", self.account)?;
        for check in &self.backends {
            match &check.error {
                None => writeln!(f, "  {}: OK", check.backend)?,
                Some(err) => writeln!(f, "  {}: FAIL ({err})", check.backend)?,
            }
        }
        Ok(())
    }
}
