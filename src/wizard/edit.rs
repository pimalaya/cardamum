//! Interactive editor for an existing account.
//!
//! Removes the named account from the merged config, runs the shared
//! account flow (see [`crate::wizard::account`]) with the account's
//! current values as defaults, then writes the updated config back.

use std::path::Path;

use anyhow::Result;

use crate::config::Config;
use crate::wizard::account;

pub fn edit_account(target: &Path, mut config: Config, account_name: &str) -> Result<Config> {
    let existing = config.accounts.remove(account_name);

    let is_first_account = config.accounts.is_empty() && existing.is_none();
    let default = existing
        .as_ref()
        .map(|a| a.default)
        .unwrap_or(is_first_account);

    let account = account::configure_existing(account_name, default, existing)?;

    config.accounts.insert(account_name.to_owned(), account);
    account::write(&config, target)?;

    Ok(config)
}
