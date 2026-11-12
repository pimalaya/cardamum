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

use crate::config::{AccountConfig, Config};

/// List all accounts declared in the configuration.
///
/// Each row names the account, the backends it declares a config block
/// for, and whether it is the default one. Nothing is reached: this
/// reads the configuration only.
///
/// JSON output: `{"accounts": [{"name", "default", "backends"}]}`.
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
    /// The backends the account declares a config block for.
    pub backends: Vec<&'static str>,
}

impl AccountRow {
    fn from_account(name: &str, account: &AccountConfig) -> Self {
        let mut backends = Vec::new();
        #[cfg(feature = "carddav")]
        if account.carddav.is_some() {
            backends.push("carddav");
        }
        #[cfg(feature = "jmap")]
        if account.jmap.is_some() {
            backends.push("jmap");
        }
        #[cfg(feature = "msgraph")]
        if account.msgraph.is_some() {
            backends.push("msgraph");
        }
        #[cfg(feature = "gpeople")]
        if account.gpeople.is_some() {
            backends.push("gpeople");
        }
        #[cfg(feature = "vdir")]
        if account.vdir.is_some() {
            backends.push("vdir");
        }
        #[cfg(feature = "pimdir")]
        if account.pimdir.is_some() {
            backends.push("pimdir");
        }

        Self {
            name: name.to_owned(),
            default: account.default,
            backends,
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
            columns.push([
                account.name.clone(),
                account.backends.join(","),
                String::from(if account.default { "yes" } else { "" }),
            ]);
        }

        write!(f, "{columns}")
    }
}
