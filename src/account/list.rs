//! # Account list command
//!
//! Lists the configured accounts as aligned columns or as JSON, without
//! reaching any backend. The output is pimalaya-cli's, shared with the
//! family.

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    account::{AccountListOutput, AccountRow},
    printer::Printer,
};
use pimalaya_config::toml::TomlConfig;

use crate::{
    backend::is_built,
    config::{AccountConfig, Config},
};

/// List all accounts declared in the configuration.
///
/// One line per account: its name, the backends it declares a config
/// block for, those this build leaves out marked `(not built)`, and
/// `default` on the one a command passing no `-a <NAME>` takes. Nothing
/// is reached: this reads the configuration only.
///
/// JSON output: `{"accounts": [{"name", "default", "backends",
/// "notBuilt"}]}`, `backends` the built ones in the order `auto` resolves
/// them, `notBuilt` the others in the same order.
#[derive(Debug, Parser)]
pub struct AccountListCommand;

impl AccountListCommand {
    pub fn execute(self, printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
        let config = Config::load(config_paths)?;

        let mut accounts: Vec<AccountRow> = config
            .accounts
            .iter()
            .map(|(name, account)| row(name, account))
            .collect();
        accounts.sort_by(|a, b| a.name.cmp(&b.name));

        printer.out(AccountListOutput { accounts })
    }
}

/// The row of the account `name`, its backends split by whether this
/// build carries them, each half in the order `auto` resolves them.
fn row(name: &str, account: &AccountConfig) -> AccountRow {
    let (backends, not_built) = account
        .backends()
        .into_iter()
        .partition(|name| is_built(name));

    AccountRow {
        name: name.to_owned(),
        default: account.default,
        backends,
        not_built,
    }
}
