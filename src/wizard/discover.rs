//! Interactive configuration wizard for first-run setup.
//!
//! Triggered by `cli::load_or_wizard` when no config file is found
//! (TomlConfig::from_paths_or_default returned `Ok(None)`).
//!
//! Flow:
//!
//! 1. Confirm with the user. Exit if they decline.
//! 2. Ask for an account name and email address.
//! 3. Ask whether the account is vdir-only, CardDAV-only, or both.
//! 4. For vdir: prompt the home directory.
//! 5. For CardDAV: prompt the discover host (defaults to the email
//!    domain) plus auth (Basic / Bearer).
//! 6. Build a Config, write it to `target`, return it.

use std::{collections::HashMap, path::Path, process::exit};

use anyhow::{Result, anyhow};
use log::info;
use pimalaya_cli::prompt;
#[cfg(feature = "carddav")]
use pimalaya_config::secret::Secret;

#[cfg(feature = "vdir")]
use crate::config::VdirConfig;
use crate::config::{AccountConfig, Config};
#[cfg(feature = "carddav")]
use crate::config::{CarddavAuthConfig, CarddavConfig};

pub fn run_or_exit(target: &Path) -> Result<Config> {
    let prompt_msg = format!(
        "No configuration found. Create one at {}?",
        target.display(),
    );

    if !prompt::bool(&prompt_msg, true)? {
        exit(0);
    }

    let account_name = prompt::text("Account name:", Some("default"))?;
    let email = prompt::text::<&str>("Email address:", None)?;

    let (_local_part, domain) = email
        .rsplit_once('@')
        .ok_or_else(|| anyhow!("Invalid email address `{email}`: missing `@`"))?;

    let account = build_account(domain)?;

    let config = Config {
        table: Default::default(),
        addressbook: Default::default(),
        card: Default::default(),
        account: Default::default(),
        accounts: HashMap::from([(account_name, account)]),
    };

    config.write(target)?;
    info!("Configuration written to {}", target.display());

    Ok(config)
}

#[allow(unused_variables)]
fn build_account(domain: &str) -> Result<AccountConfig> {
    #[cfg(feature = "vdir")]
    let want_vdir = prompt::bool("Configure a local vdir backend?", false)?;
    #[cfg(not(feature = "vdir"))]
    let want_vdir = false;

    #[cfg(feature = "carddav")]
    let want_carddav = prompt::bool("Configure a remote CardDAV (CardDAV) backend?", true)?;
    #[cfg(not(feature = "carddav"))]
    let want_carddav = false;

    if !want_vdir && !want_carddav {
        return Err(anyhow!(
            "At least one backend (vdir or carddav) must be configured"
        ));
    }

    #[cfg(feature = "vdir")]
    let vdir = if want_vdir {
        let home_dir = prompt::text::<&str>("Vdir home directory:", None)?;
        Some(VdirConfig { home_dir })
    } else {
        None
    };

    #[cfg(feature = "carddav")]
    let carddav = if want_carddav {
        Some(prompt_carddav(domain)?)
    } else {
        None
    };

    Ok(AccountConfig {
        default: true,
        table: Default::default(),
        addressbook: Default::default(),
        card: Default::default(),
        #[cfg(feature = "vdir")]
        vdir,
        #[cfg(feature = "carddav")]
        carddav,
    })
}

#[cfg(feature = "carddav")]
fn prompt_carddav(domain: &str) -> Result<CarddavConfig> {
    let discover = prompt::text("Discover domain:", Some(domain))?;

    let auth_kind = prompt::item(
        "Authentication scheme:",
        ["basic".to_string(), "bearer".to_string()],
        Some("basic".to_string()),
    )?;

    let auth = match auth_kind.as_str() {
        "bearer" => {
            let token = prompt::secret("Bearer token:")?;
            CarddavAuthConfig::Bearer {
                token: Secret::Raw(token.into()),
            }
        }
        _ => {
            let username = prompt::text::<&str>("Username:", None)?;
            let password = prompt::secret("Password:")?;
            CarddavAuthConfig::Basic {
                username,
                password: Secret::Raw(password.into()),
            }
        }
    };

    Ok(CarddavConfig {
        discover: Some(discover),
        server: None,
        home: None,
        tls: Default::default(),
        auth,
    })
}
