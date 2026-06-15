//! Interactive configuration wizard for first-run setup.
//!
//! Triggered by `cli::load_or_wizard` when no config file is found
//! (TomlConfig::from_paths_or_default returned `Ok(None)`).
//!
//! Confirms with the user, asks for an account name, then runs the
//! shared account flow (see [`crate::wizard::account`]) with no
//! existing defaults, and writes the result.

use std::{collections::HashMap, path::Path, process::exit};

use anyhow::Result;
use pimalaya_cli::prompt;

use crate::config::Config;
use crate::wizard::account;

pub fn run_or_exit(target: &Path) -> Result<Config> {
    let prompt_msg = format!(
        "No configuration found. Create one at {}?",
        target.display()
    );

    if !prompt::bool(&prompt_msg, true)? {
        exit(0);
    }

    let account_name = prompt::text("Account name:", Some("personal"))?;
    let account = account::configure(&account_name, true, None)?;

    let config = Config {
        accounts: HashMap::from([(account_name, account)]),
        ..Default::default()
    };

    account::write(&config, target)?;

    Ok(config)
}
