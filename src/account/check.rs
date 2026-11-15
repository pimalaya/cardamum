//! # Account check command
//!
//! Reaches every backend the account configures and reports, one by one,
//! whether it answered.
//!
//! The backends of one account resolve their credentials through the
//! same secret resolver, so a command two of them name is spawned once.

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::{
    account::{AccountCheckOutput, BackendCheck},
    error::{PartialOutcome, with_code},
    printer::Printer,
};
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
use pimalaya_config::secret::SecretResolver;
use pimalaya_config::toml::TomlConfig;

use crate::{
    backend::{Backend, NAMES, is_built, not_built},
    config::Config,
    error::{self, ErrorCode},
};

/// Validate the account configuration.
///
/// Picks the active account, from the global `--account` flag or from
/// the default, then reaches every backend `--backend` allows. Each
/// check walks the same handshake and authentication path the other
/// commands take, so a bad credential surfaces here. Under `auto`, a
/// backend the build leaves out fails coded `backend-not-built`.
///
/// The report is printed whole, each backend with its outcome and, when
/// it failed, its code and why. A failed backend exits the command with
/// status 3, the report being the output rather than a failure object.
///
/// JSON output: `{"account", "backends": [{"backend", "ok", "code",
/// "error"}]}`.
#[derive(Debug, Parser)]
pub struct AccountCheckCommand;

impl AccountCheckCommand {
    #[cfg_attr(
        not(any(
            feature = "vdir",
            feature = "pimdir",
            feature = "carddav",
            feature = "jmap",
            feature = "msgraph",
            feature = "gpeople"
        )),
        allow(unused_mut, unused_variables)
    )]
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        let mut config = Config::load(config_paths)?;
        let (name, account_config) = config.take_account(account_name)?;

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
                .push(backend_check("vdir", connect_vdir(vdir_config)));
        }

        #[cfg(feature = "pimdir")]
        if backend.allows_pimdir()
            && let Some(pimdir_config) = &account_config.pimdir
        {
            report
                .backends
                .push(backend_check("pimdir", connect_pimdir(pimdir_config)));
        }

        #[cfg(feature = "carddav")]
        if backend.allows_carddav()
            && let Some(carddav_config) = &account_config.carddav
        {
            report.backends.push(backend_check(
                "carddav",
                connect_carddav(carddav_config, &mut resolver),
            ));
        }

        #[cfg(feature = "jmap")]
        if backend.allows_jmap()
            && let Some(jmap_config) = &account_config.jmap
        {
            report.backends.push(backend_check(
                "jmap",
                connect_jmap(jmap_config, &mut resolver),
            ));
        }

        #[cfg(feature = "msgraph")]
        if backend.allows_msgraph()
            && let Some(msgraph_config) = &account_config.msgraph
        {
            report.backends.push(backend_check(
                "msgraph",
                connect_msgraph(msgraph_config, &mut resolver),
            ));
        }

        #[cfg(feature = "gpeople")]
        if backend.allows_gpeople()
            && let Some(gpeople_config) = &account_config.gpeople
        {
            report.backends.push(backend_check(
                "gpeople",
                connect_gpeople(gpeople_config, &mut resolver),
            ));
        }

        // NOTE: under `auto`, a backend the build leaves out is reported
        // rather than skipped, at its place in the resolution order.
        if backend == Backend::Auto {
            let names = account_config.backends().into_iter();
            for name in names.filter(|name| !is_built(name)) {
                let check = backend_check(name, Err(not_built(name)));
                report.backends.push(check);
            }
            report
                .backends
                .sort_by_key(|check| NAMES.iter().position(|name| *name == check.backend));
        }

        if report.backends.is_empty() {
            bail!("No backend matching `{backend}` is configured for this account");
        }

        let failed = report.backends.iter().any(|check| !check.ok);
        printer.out(report)?;

        // NOTE: the report names every backend's outcome, so a failed one
        // ends the command with the partial-outcome status rather than a
        // failure object.
        if failed {
            return Err(PartialOutcome::default().into());
        }

        Ok(())
    }
}

/// Checks the vdir home is a directory, a local tree having nothing to
/// connect to.
#[cfg(feature = "vdir")]
fn connect_vdir(vdir_config: &crate::config::VdirConfig) -> Result<()> {
    use std::path::Path;

    let home = Path::new(&vdir_config.root);
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
/// That exercises the index the unified commands read through and the
/// staging lock their writes take, a local store having no handshake nor
/// authentication to check.
#[cfg(feature = "pimdir")]
fn connect_pimdir(pimdir_config: &crate::config::PimdirConfig) -> Result<()> {
    use crate::pimdir::backend::PimdirBackend;

    PimdirBackend::new(pimdir_config.clone())?.check()
}

/// Resolves the CardDAV context root, then walks the principal and the
/// addressbook home set, proving address, TLS and authentication work.
///
/// A `home` block skips that walk, so the check sends the home one
/// `PROPFIND` of depth 0 instead, which a refused credential fails.
#[cfg(feature = "carddav")]
fn connect_carddav(
    carddav_config: &crate::config::CarddavConfig,
    resolver: &mut SecretResolver,
) -> Result<()> {
    use crate::carddav::client::{open_carddav_client, propfind_home};

    let mut client = open_carddav_client(carddav_config.clone(), resolver)?;

    // NOTE: the `home` route connects without a request, so nothing it did
    // proves the credential. The other commands need no such request, their
    // own first one being authenticated.
    if carddav_config.home.is_some() {
        propfind_home(&mut client)?;
    }

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

/// The row of `backend` in the report, coded as the failure object of any
/// other command would be, a code read off a typed error included.
fn backend_check(backend: &'static str, result: Result<()>) -> BackendCheck<ErrorCode> {
    BackendCheck::new(backend, result.map_err(|err| with_code(err, error::code)))
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

        // NOTE: the report carries the code too, as the failure object
        // of any other command would.
        let check = backend_check("pimdir", connect_pimdir(&config));
        let check = serde_json::to_value(check).unwrap();
        assert_eq!(check["code"], "store-busy", "{check}");

        drop(collector);
        let check = backend_check("pimdir", connect_pimdir(&config));
        let check = serde_json::to_value(check).unwrap();
        assert_eq!(check.get("code"), Some(&serde_json::Value::Null), "{check}");

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
                    "[accounts.test]\ndefault = true\nvdir.root = {:?}\n",
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
        let vdir = out.lines().find(|line| line.starts_with("vdir:"));
        assert_eq!(
            vdir.map(|line| line.split_whitespace().last()),
            Some(Some("OK")),
            "{out}"
        );

        // NOTE: the report still prints, the failure only setting the
        // partial-outcome exit status.
        let (result, out) = check(&dir.path().join("missing"));
        let err = result.unwrap_err();
        let partial = pimalaya_cli::error::partial_outcome_of(&err);
        assert_eq!(partial.map(|partial| partial.status()), Some(3), "{err:#}");
        let vdir = out.lines().find(|line| line.starts_with("vdir:"));
        let vdir = vdir.unwrap_or_default();
        assert!(
            vdir.contains("FAIL (") && vdir.contains("does not exist"),
            "{out}"
        );
    }
}

#[cfg(all(test, feature = "carddav"))]
mod carddav_tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use pimalaya_config::secret::{Secret, SecretResolver};
    use secrecy::SecretString;
    use serde_json::{Value, json};

    use super::{backend_check, connect_carddav};
    use crate::config::{CarddavAuthConfig, CarddavConfig};

    /// A scripted server answering every request with `status` and no
    /// body, a connection that sends none answered nothing. Returns its
    /// URL, whose path is the address book home and the context root.
    fn refusing(status: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { return };

                if read_request(&mut stream) {
                    let response = format!(
                        "HTTP/1.1 {status}\r\nContent-Length: 0\r\n\
                         Connection: close\r\n\r\n"
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        });

        format!("http://127.0.0.1:{port}/dav/home/")
    }

    /// Reads one request, its head and the body its `Content-Length`
    /// names; `false` when the connection closed before one came.
    fn read_request(stream: &mut impl Read) -> bool {
        let mut request = Vec::new();
        let mut buf = [0u8; 1024];

        let head = loop {
            if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                break end + 4;
            }
            match stream.read(&mut buf) {
                Ok(0) | Err(_) => return false,
                Ok(n) => request.extend_from_slice(&buf[..n]),
            }
        };

        let length = String::from_utf8_lossy(&request[..head])
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if !name.eq_ignore_ascii_case("content-length") {
                    return None;
                }
                value.trim().parse().ok()
            })
            .unwrap_or(0);

        while request.len() < head + length {
            match stream.read(&mut buf) {
                Ok(0) | Err(_) => return false,
                Ok(n) => request.extend_from_slice(&buf[..n]),
            }
        }

        true
    }

    /// A CardDAV block over `server` or `home`, its bearer token made up.
    fn config(server: Option<String>, home: Option<&str>) -> CarddavConfig {
        CarddavConfig {
            server,
            home: home.map(|home| home.parse().unwrap()),
            tls: Default::default(),
            alpn: vec![String::from("http/1.1")],
            proxy: None,
            auth: CarddavAuthConfig::Bearer {
                token: Secret::Raw(SecretString::from("made-up")),
            },
            discover: None,
        }
    }

    /// The `account check` row of `config`, as JSON.
    fn check(config: &CarddavConfig) -> Value {
        let result = connect_carddav(config, &mut SecretResolver::new());
        serde_json::to_value(backend_check("carddav", result)).unwrap()
    }

    /// A `home` block sends nothing while connecting, so the check sends
    /// it a request: a refused credential fails it with the code and the
    /// error a `server` block fails with.
    #[test]
    fn a_refused_credential_fails_a_home_check_as_a_server_one() {
        let refusals = [
            ("401 Unauthorized", "HTTP 401", Value::Null),
            ("403 Forbidden", "HTTP 403", json!("permission-denied")),
        ];

        for (status, error, code) in refusals {
            let url = refusing(status);
            let home = check(&config(None, Some(&url)));
            let server = check(&config(Some(url), None));

            assert_eq!(home["ok"], false, "{home}");
            assert_eq!(home["code"], code, "{home}");
            let message = home["error"].as_str().unwrap_or_default();
            assert!(message.contains(error), "{home}");

            assert_eq!(server["ok"], false, "{server}");
            assert_eq!(home["code"], server["code"], "{home} {server}");
            assert_eq!(home["error"], server["error"], "{home} {server}");
        }
    }
}
