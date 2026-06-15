//! Shared CardDAV wizard.
//!
//! A single discovery flow: collects authentication, then opens the
//! client through the `discover` route (which resolves the server via
//! PACC/RFC 6764, or Google's authenticated `.well-known`), and on
//! success persists the resolved `home` so later runs skip discovery. A
//! failed connection re-runs the prompts. Used both to create an
//! account and to re-configure an existing one.

use anyhow::{Result, anyhow};
use pimalaya_cli::{
    spinner::Spinner,
    wizard::carddav::{self as carddav_wizard, CarddavAuth, CarddavSecret, WizardCarddavConfig},
};
use pimalaya_config::{command::shell, secret::Secret};

use crate::{
    carddav::client::{is_google, open_carddav_client},
    config::{CarddavAuthConfig, CarddavConfig},
};

/// Runs the CardDAV wizard for `account_name` until the connection
/// succeeds, then returns a [`CarddavConfig`] holding the resolved
/// `home`. Discovery always runs (the connection test resolves the
/// server from `discover`), so this one flow both creates and
/// re-configures accounts; `existing` only seeds the auth defaults.
pub fn configure(
    project_name: &str,
    account_name: &str,
    email: &str,
    existing: Option<&CarddavConfig>,
) -> Result<CarddavConfig> {
    let (_local_part, domain) = email
        .rsplit_once('@')
        .ok_or_else(|| anyhow!("Invalid email address `{email}`: missing `@`"))?;

    let tls_config = existing.map(|e| e.tls.clone()).unwrap_or_default();

    let auth = match existing.map(|e| &e.auth) {
        Some(CarddavAuthConfig::Basic { username, password }) => CarddavAuth::Basic {
            username: username.clone(),
            secret: secret_to_wizard(password),
        },
        Some(CarddavAuthConfig::Bearer { token }) => CarddavAuth::Bearer {
            secret: secret_to_wizard(token),
        },
        // Google only accepts OAuth 2.0, so default a fresh account to
        // Bearer (the strategy prompt drops Basic via `bearer_only`).
        None if is_google(domain) => CarddavAuth::Bearer {
            secret: CarddavSecret::default(),
        },
        None => CarddavAuth::default(),
    };

    let mut defaults = WizardCarddavConfig {
        account_name: account_name.to_owned(),
        project_name: project_name.to_owned(),
        email: Some(email.to_owned()),
        auth,
        bearer_only: is_google(domain),
    };

    loop {
        let cfg = carddav_wizard::run(&defaults)?;

        // Single flow: always take the `discover` route. It resolves the
        // server (PACC/RFC 6764, or Google's authenticated
        // `.well-known`) using the just-collected auth.
        let config = CarddavConfig {
            discover: Some(domain.to_owned()),
            server: None,
            home: None,
            tls: tls_config.clone(),
            auth: carddav_auth_to_config(cfg.auth.clone()),
        };

        let spinner = Spinner::start("Testing connection");

        match open_carddav_client(config.clone()) {
            Ok(client) => {
                spinner.success("Connection successful");

                // Persist the resolved home-set so later runs skip
                // discovery entirely (the `home` route short-circuits it).
                return Ok(match client.addressbook_home_set.clone() {
                    Some(home) => CarddavConfig {
                        discover: None,
                        server: None,
                        home: Some(home),
                        ..config
                    },
                    None => config,
                });
            }
            Err(err) => {
                spinner.failure(format!("Connection failed: {err}"));
                defaults = cfg;
            }
        }
    }
}

fn carddav_auth_to_config(auth: CarddavAuth) -> CarddavAuthConfig {
    match auth {
        CarddavAuth::Basic { username, secret } => CarddavAuthConfig::Basic {
            username,
            password: carddav_secret_to_secret(secret),
        },
        CarddavAuth::Bearer { secret } => CarddavAuthConfig::Bearer {
            token: carddav_secret_to_secret(secret),
        },
    }
}

fn carddav_secret_to_secret(secret: CarddavSecret) -> Secret {
    match secret {
        CarddavSecret::Raw(s) => Secret::Raw(s),
        CarddavSecret::Command(cmd) => Secret::Command(shell(&cmd)),
    }
}

/// Carries an existing secret into the wizard so editing prefills it: a
/// `Command` keeps its command line (defaulting the secret strategy to
/// shell command), a `Raw` keeps only the strategy.
fn secret_to_wizard(secret: &Secret) -> CarddavSecret {
    match secret_command_line(secret) {
        Some(line) => CarddavSecret::Command(line),
        None => CarddavSecret::Raw(String::new().into()),
    }
}

/// Reconstructs the command line behind a `Secret::Command`: unwraps the
/// platform-shell form produced by `shell`, otherwise rejoins program +
/// args. Returns `None` for a `Secret::Raw`.
fn secret_command_line(secret: &Secret) -> Option<String> {
    let Secret::Command(cmd) = secret else {
        return None;
    };

    let program = cmd.get_program().to_string_lossy().into_owned();
    let args: Vec<String> = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    let (shell_program, shell_flag) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("/bin/sh", "-c")
    };

    if program == shell_program {
        if let [flag, line] = args.as_slice() {
            if flag == shell_flag {
                return Some(line.clone());
            }
        }
    }

    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(program);
    parts.extend(args);
    Some(parts.join(" "))
}
