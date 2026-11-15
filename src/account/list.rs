//! # Account list command
//!
//! Lists the configured accounts as aligned columns or as JSON, without
//! reaching any backend.

use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{columns::Columns, printer::Printer};
use pimalaya_config::toml::TomlConfig;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    backend::is_built,
    config::{AccountConfig, Config},
};

/// List all accounts declared in the configuration.
///
/// Each row names the account, the backends it declares a config block
/// for, marking those this build leaves out, and whether it is the
/// default one. Nothing is reached: this reads the configuration only.
///
/// JSON output: `{"accounts": [{"name", "default", "backends",
/// "notBuilt"}]}`.
#[derive(Debug, Parser)]
pub struct AccountListCommand;

impl AccountListCommand {
    pub fn execute(self, printer: &mut impl Printer, config_paths: &[PathBuf]) -> Result<()> {
        let config = Config::load(config_paths)?;

        let mut accounts: Vec<AccountRow> = config
            .accounts
            .iter()
            .map(|(name, account)| AccountRow::from_account(name, account))
            .collect();
        accounts.sort_by(|a, b| a.name.cmp(&b.name));

        printer.out(AccountListOutput { accounts })
    }
}

/// One account, as rendered by [`AccountListOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountRow {
    /// The `[accounts.<name>]` table key.
    pub name: String,
    /// Whether a command passing no `-a <NAME>` runs against it.
    pub default: bool,
    /// The backends the account declares a config block for, built or
    /// not, in the order `auto` resolves them.
    pub backends: Vec<&'static str>,
    /// Those of `backends` this build leaves out, which a command
    /// reaching fails coded `backend-not-built`.
    pub not_built: Vec<&'static str>,
}

impl AccountRow {
    fn from_account(name: &str, account: &AccountConfig) -> Self {
        let backends = account.backends();
        let not_built = backends
            .iter()
            .copied()
            .filter(|name| !is_built(name))
            .collect();

        Self {
            name: name.to_owned(),
            default: account.default,
            backends,
            not_built,
        }
    }
}

/// The accounts, and the JSON shape `account list` prints.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountListOutput {
    /// The accounts, sorted by name.
    pub accounts: Vec<AccountRow>,
}

impl fmt::Display for AccountListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::new(["NAME", "BACKENDS", "DEFAULT"]);

        for account in &self.accounts {
            let backends: Vec<String> = account
                .backends
                .iter()
                .map(|name| match account.not_built.contains(name) {
                    true => format!("{name} (not built)"),
                    false => name.to_string(),
                })
                .collect();

            columns.push([
                account.name.clone(),
                backends.join(","),
                String::from(if account.default { "yes" } else { "" }),
            ]);
        }

        write!(f, "{columns}")
    }
}
