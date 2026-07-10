//! Shared CardDAV wizard.
//!
//! Two entry points, one per way the endpoint is known. The user typed a
//! server URL: [`configure_server`] prompts the authentication, then
//! opens the client through the `server` route. Discovery pinned the
//! endpoint and the authentication method: [`configure_discovered`]
//! prompts only the credentials. Both persist the resolved `home` on
//! success so later runs skip discovery, and re-run the prompts on a
//! failed connection.

use anyhow::Result;
use pimalaya_cli::{
    prompt,
    spinner::Spinner,
    wizard::carddav::{self as carddav_wizard, CarddavAuth, CarddavSecret, WizardCarddavConfig},
};
use pimalaya_config::{command::shell, secret::Secret};
use url::Url;

use crate::{
    carddav::client::open_carddav_client,
    config::{CarddavAuthConfig, CarddavConfig},
    wizard::{
        search::{Discovered, DiscoveredAuth},
        secret,
    },
};

/// Runs the CardDAV wizard against a known `server` URL until the
/// connection succeeds, then returns a [`CarddavConfig`] holding the
/// resolved `home`. `existing` seeds the auth defaults when editing.
pub fn configure_server(
    project_name: &str,
    account_name: &str,
    server: &Url,
    existing: Option<&CarddavConfig>,
) -> Result<CarddavConfig> {
    let tls_config = existing.map(|e| e.tls.clone()).unwrap_or_default();

    let auth = match existing.map(|e| &e.auth) {
        Some(CarddavAuthConfig::Basic { username, password }) => CarddavAuth::Basic {
            username: username.clone(),
            secret: secret_to_wizard(password),
        },
        Some(CarddavAuthConfig::Bearer { token }) => CarddavAuth::Bearer {
            secret: secret_to_wizard(token),
        },
        None => CarddavAuth::default(),
    };

    let mut defaults = WizardCarddavConfig {
        account_name: account_name.to_owned(),
        project_name: project_name.to_owned(),
        email: None,
        auth,
        bearer_only: false,
    };

    loop {
        let cfg = carddav_wizard::run(&defaults)?;

        let config = CarddavConfig {
            discover: None,
            server: Some(server.to_string()),
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

/// Runs the CardDAV wizard for one discovered entry: the endpoint and
/// the authentication method are pinned by the discovery, so only the
/// credentials are prompted. On success the resolved home-set is
/// persisted, like the manual flow.
pub fn configure_discovered(
    email: &str,
    url: &Url,
    discovered: &Discovered,
) -> Result<CarddavConfig> {
    loop {
        let auth = match discovered.auth {
            DiscoveredAuth::Password => {
                let default_username = discovered
                    .username
                    .clone()
                    .unwrap_or_else(|| email.to_string());
                let username = prompt::text("CardDAV username:", Some(default_username.as_str()))?;
                let password = secret::configure("CardDAV password", None)?;
                CarddavAuthConfig::Basic { username, password }
            }
            DiscoveredAuth::Token => {
                let token = secret::configure("CardDAV bearer token", Some("ortie token show"))?;
                CarddavAuthConfig::Bearer { token }
            }
        };

        let config = CarddavConfig {
            discover: None,
            server: Some(url.to_string()),
            home: None,
            tls: Default::default(),
            auth,
        };

        let spinner = Spinner::start("Testing connection");

        match open_carddav_client(config.clone()) {
            Ok(client) => {
                spinner.success("Connection successful");

                return Ok(match client.addressbook_home_set.clone() {
                    Some(home) => CarddavConfig {
                        server: None,
                        home: Some(home),
                        ..config
                    },
                    None => config,
                });
            }
            Err(err) => spinner.failure(format!("Connection failed: {err}")),
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
