//! # Account check command
//!
//! Reaches every backend the account configures and reports, one by one,
//! whether it answered.
//!
//! The backends of one account resolve their credentials through the
//! same secret resolver, so a command two of them name is spawned once.

use std::{fmt, path::PathBuf};

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use pimalaya_config::secret::SecretResolver;
use pimalaya_config::toml::TomlConfig;
use schemars::JsonSchema;
use serde::Serialize;

#[cfg(feature = "wizard")]
use crate::config::AccountConfig;
use crate::{
    backend::Backend,
    config::{Config, NO_CONFIG_HINT},
};

/// Validate the account configuration.
///
/// Picks the active account, from the global `--account` flag or from
/// the default, then reaches every backend `--backend` allows. Each
/// check walks the same handshake and authentication path the other
/// commands take, so a bad credential surfaces here.
///
/// A backend that fails fails the command, with one error naming each
/// failed backend and why.
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
                "No configuration found at {}, {NO_CONFIG_HINT}",
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

        let mut report = AccountCheckOutput {
            account: name,
            backends: Vec::new(),
        };

        #[cfg(any(
            feature = "carddav",
            feature = "jmap",
            feature = "msgraph",
            feature = "gpeople"
        ))]
        let mut resolver = SecretResolver::new();

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
                connect_carddav(carddav_config, &mut resolver),
            ));
        }

        #[cfg(feature = "jmap")]
        if backend.allows_jmap()
            && let Some(jmap_config) = &account_config.jmap
        {
            report.backends.push(BackendCheck::from(
                "jmap",
                connect_jmap(jmap_config, &mut resolver),
            ));
        }

        #[cfg(feature = "msgraph")]
        if backend.allows_msgraph()
            && let Some(msgraph_config) = &account_config.msgraph
        {
            report.backends.push(BackendCheck::from(
                "msgraph",
                connect_msgraph(msgraph_config, &mut resolver),
            ));
        }

        #[cfg(feature = "gpeople")]
        if backend.allows_gpeople()
            && let Some(gpeople_config) = &account_config.gpeople
        {
            report.backends.push(BackendCheck::from(
                "gpeople",
                connect_gpeople(gpeople_config, &mut resolver),
            ));
        }

        if report.backends.is_empty() {
            bail!("No backend matching `{backend}` is configured for this account");
        }

        let failed: Vec<String> = report
            .backends
            .iter()
            .filter_map(|check| {
                let err = check.error.as_ref()?;
                Some(format!("{}: {err}", check.backend))
            })
            .collect();

        if !failed.is_empty() {
            bail!(
                "Account `{}` failed its check: {}",
                report.account,
                failed.join("; ")
            );
        }

        printer.out(report)
    }
}

/// Tests every backend the account configures, failing on the first error.
///
/// The wizard runs it over a freshly built account, so a bad credential
/// or endpoint stops the process instead of yielding a configuration
/// that cannot connect.
#[cfg(feature = "wizard")]
pub fn test_account(account_config: &AccountConfig) -> Result<()> {
    #[cfg(any(
        feature = "carddav",
        feature = "jmap",
        feature = "msgraph",
        feature = "gpeople"
    ))]
    let mut resolver = SecretResolver::new();

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
        connect_carddav(carddav_config, &mut resolver)?;
    }

    #[cfg(feature = "jmap")]
    if let Some(jmap_config) = &account_config.jmap {
        connect_jmap(jmap_config, &mut resolver)?;
    }

    #[cfg(feature = "msgraph")]
    if let Some(msgraph_config) = &account_config.msgraph {
        connect_msgraph(msgraph_config, &mut resolver)?;
    }

    #[cfg(feature = "gpeople")]
    if let Some(gpeople_config) = &account_config.gpeople {
        connect_gpeople(gpeople_config, &mut resolver)?;
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

/// Opens the pimdir store, reads its collection list and probes a producer.
///
/// That exercises the index the shared commands read through and the
/// staging lock their writes take, a local store having no handshake nor
/// authentication to check.
#[cfg(feature = "pimdir")]
fn connect_pimdir(pimdir_config: &crate::config::PimdirConfig) -> Result<()> {
    use crate::pimdir::backend::PimdirBackend;

    PimdirBackend::new(pimdir_config.clone())?.check()
}

/// Resolves the CardDAV context root, then walks the principal and the
/// addressbook home set, proving address, TLS and authentication work.
#[cfg(feature = "carddav")]
fn connect_carddav(
    carddav_config: &crate::config::CarddavConfig,
    resolver: &mut SecretResolver,
) -> Result<()> {
    use crate::carddav::client::open_carddav_client;

    open_carddav_client(carddav_config.clone(), resolver)?;

    Ok(())
}

/// Establishes the JMAP session, proving address, TLS and authentication
/// work.
#[cfg(feature = "jmap")]
fn connect_jmap(
    jmap_config: &crate::config::JmapConfig,
    resolver: &mut SecretResolver,
) -> Result<()> {
    use crate::jmap::backend::JmapBackend;

    JmapBackend::new(jmap_config.clone(), resolver)?;

    Ok(())
}

/// Lists the Graph contact folders, proving the token grants access to
/// the contacts API.
#[cfg(feature = "msgraph")]
fn connect_msgraph(
    msgraph_config: &crate::config::MsgraphConfig,
    resolver: &mut SecretResolver,
) -> Result<()> {
    use crate::msgraph::backend::MsgraphBackend;

    let mut client = MsgraphBackend::new(msgraph_config.clone(), resolver)?;
    client.list_addressbooks()?;

    Ok(())
}

/// Lists the People contact groups, proving the token grants access to
/// the contacts API.
#[cfg(feature = "gpeople")]
fn connect_gpeople(
    gpeople_config: &crate::config::GpeopleConfig,
    resolver: &mut SecretResolver,
) -> Result<()> {
    use crate::gpeople::backend::GpeopleBackend;

    let mut client = GpeopleBackend::new(gpeople_config.clone(), resolver)?;
    client.list_addressbooks()?;

    Ok(())
}

/// Result of a check: one outcome per configured backend.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountCheckOutput {
    /// The account that was checked.
    pub account: String,
    /// One outcome per backend the account configures.
    pub backends: Vec<BackendCheck>,
}

/// Outcome of checking a single backend.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BackendCheck {
    /// The backend that was reached.
    pub backend: &'static str,
    /// Whether it answered.
    pub ok: bool,
    /// Why it did not, when it did not.
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

impl fmt::Display for AccountCheckOutput {
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

#[cfg(all(test, feature = "pimdir"))]
mod tests {
    use super::*;

    /// A store a reader opens but whose staging lock a producer cannot
    /// take fails the check rather than the first write.
    #[test]
    fn a_store_no_producer_opens_fails_the_check() {
        let dir = tempfile::tempdir().unwrap();
        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .ensure_collection("book", "text/vcard")
            .unwrap();
        let lock = dir.path().join("objects.lock");
        let _ = std::fs::remove_file(&lock);
        std::fs::create_dir(&lock).unwrap();

        let config = crate::config::PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };
        assert!(connect_pimdir(&config).is_err());
    }

    /// The check takes the staging lock through a producer probe, which
    /// leaves no file in the blob tree.
    #[test]
    fn a_check_leaves_no_body_behind() {
        let dir = tempfile::tempdir().unwrap();
        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .ensure_collection("book", "text/vcard")
            .unwrap();

        let config = crate::config::PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };
        connect_pimdir(&config).unwrap();

        let objects = dir.path().join("objects");
        let left: Vec<_> = match std::fs::read_dir(&objects) {
            Ok(entries) => entries.map(|entry| entry.unwrap().path()).collect(),
            Err(_) => Vec::new(),
        };
        assert!(left.is_empty(), "{left:?}");
    }

    /// A collector holding the staging lock fails the check at once with
    /// `store-busy`, rather than leaving it waiting, and the check stages
    /// nothing into the blob tree.
    #[test]
    fn a_held_staging_lock_fails_the_check_as_busy() {
        let dir = tempfile::tempdir().unwrap();
        io_pimdir::client::PimdirStore::open(dir.path())
            .unwrap()
            .ensure_collection("book", "text/vcard")
            .unwrap();
        let collector = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dir.path().join("objects.lock"))
            .unwrap();
        collector.try_lock().unwrap();

        let config = crate::config::PimdirConfig {
            root: dir.path().to_path_buf(),
            account: None,
        };
        let err = connect_pimdir(&config).unwrap_err();
        assert_eq!(
            pimalaya_cli::error::code_of(&err),
            Some(crate::error::ErrorCode::StoreBusy),
            "{err:#}"
        );

        let objects = dir.path().join("objects");
        let left: Vec<_> = match std::fs::read_dir(&objects) {
            Ok(entries) => entries.map(|entry| entry.unwrap().path()).collect(),
            Err(_) => Vec::new(),
        };
        assert!(left.is_empty(), "{left:?}");
    }
}

#[cfg(all(test, feature = "vdir"))]
mod vdir_tests {
    use core::fmt;
    use std::path::Path;

    use anyhow::Result;
    use pimalaya_cli::printer::Printer;
    use serde::Serialize;

    use crate::{account::check::AccountCheckCommand, backend::Backend};

    /// A printer keeping what it is handed, as text.
    #[derive(Default)]
    struct Capture(String);

    impl Printer for Capture {
        fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
            self.0.push_str(&data.to_string());
            Ok(())
        }

        fn event<T: std::fmt::Display + serde::Serialize>(&mut self, _event: T) -> Result<()> {
            Ok(())
        }

        fn out_bytes(&mut self, _bytes: pimalaya_cli::output::Bytes) -> Result<()> {
            Ok(())
        }

        fn is_json(&self) -> bool {
            false
        }
    }

    /// A script reads the verdict from the exit code, so a backend that
    /// fails fails the command, naming it and why.
    #[test]
    fn a_failed_backend_fails_the_check() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config.toml");
        let check = |home: &Path| {
            std::fs::write(
                &config,
                format!(
                    "[accounts.test]\ndefault = true\nvdir.home-dir = {:?}\n",
                    home.display().to_string(),
                ),
            )
            .unwrap();
            let mut printer = Capture::default();
            let result = AccountCheckCommand.execute(
                &mut printer,
                core::slice::from_ref(&config),
                None,
                Backend::Auto,
            );
            (result, printer.0)
        };

        let (result, out) = check(dir.path());
        result.unwrap();
        assert!(out.contains("vdir: OK"), "{out}");

        let (result, out) = check(&dir.path().join("missing"));
        let err = format!("{:#}", result.unwrap_err());
        assert!(err.contains("vdir"), "{err}");
        assert!(err.contains("does not exist"), "{err}");
        assert!(out.is_empty(), "{out}");
    }
}
