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
/// Loads the TOML configuration, picks the active account (via the
/// global `--account` flag or the default), and checks each backend
/// allowed by `--backend`.
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
                "No configuration found. Run `cardamum` once to launch the wizard, \
                 or `cardamum account configure <name>` to create one."
            ),
        };

        let (name, account_config) = config
            .take_account(account_name)?
            .ok_or_else(|| anyhow::anyhow!("Cannot find account"))?;

        let mut report = CheckReport {
            account: name,
            backends: Vec::new(),
        };

        #[cfg(feature = "vdir")]
        if backend.allows_vdir() {
            if let Some(vdir_config) = account_config.vdir.clone() {
                report
                    .backends
                    .push(check_vdir(&config, &account_config, vdir_config));
            }
        }

        #[cfg(feature = "carddav")]
        if backend.allows_carddav() {
            if let Some(carddav_config) = account_config.carddav.clone() {
                report
                    .backends
                    .push(check_carddav(&config, &account_config, carddav_config));
            }
        }

        if report.backends.is_empty() {
            bail!("No backend matching `{backend}` is configured for this account");
        }

        printer.out(report)
    }
}

#[cfg(feature = "vdir")]
fn check_vdir(
    _config: &Config,
    _account_config: &AccountConfig,
    vdir_config: crate::config::VdirConfig,
) -> BackendCheck {
    use std::path::Path;

    let result = (|| -> Result<()> {
        let home = Path::new(&vdir_config.home_dir);
        if !home.is_dir() {
            bail!(
                "Vdir home `{}` does not exist or is not a directory",
                home.display()
            );
        }
        Ok(())
    })();

    BackendCheck::from("vdir", result)
}

#[cfg(feature = "carddav")]
fn check_carddav(
    _config: &Config,
    _account_config: &AccountConfig,
    carddav_config: crate::config::CarddavConfig,
) -> BackendCheck {
    use crate::carddav::client::open_carddav_client;

    let result = (|| -> Result<()> {
        let _client = open_carddav_client(carddav_config)?;
        Ok(())
    })();

    BackendCheck::from("carddav", result)
}

#[derive(Clone, Debug, Serialize)]
pub struct CheckReport {
    pub account: String,
    pub backends: Vec<BackendCheck>,
}

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
