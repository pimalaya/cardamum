//! # Account list command
//!
//! Lists the configured accounts as a colored table or as JSON, without
//! reaching any backend.

use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, ContentArrangement, Row, Table};
use crossterm::style::Color as CrosstermColor;
use pimalaya_cli::printer::Printer;
use pimalaya_config::toml::TomlConfig;
use serde::Serialize;

use crate::{
    account::context::map_color_or,
    config::{AccountConfig, Config, TableArrangementConfig},
    shared::table::{DEFAULT_PRESET, style_from_preset},
};

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
        let config = load_config(config_paths)?;

        let preset = config
            .table
            .preset
            .clone()
            .unwrap_or_else(|| DEFAULT_PRESET.to_string());
        let arrangement = config
            .table
            .arrangement
            .clone()
            .unwrap_or(TableArrangementConfig::Dynamic)
            .into();

        let table_cfg = &config.account.list.table;
        let colors = AccountColors {
            name: map_color_or(table_cfg.name_color, CrosstermColor::Green),
            backends: map_color_or(table_cfg.backends_color, CrosstermColor::Blue),
            default: map_color_or(table_cfg.default_color, CrosstermColor::Reset),
        };

        let mut accounts: Vec<AccountRow> = config
            .accounts
            .iter()
            .map(|(name, account)| AccountRow::from_account(name, account))
            .collect();
        accounts.sort_by(|a, b| a.name.cmp(&b.name));

        let table = AccountsTable {
            preset,
            arrangement,
            colors,
            accounts,
        };

        printer.out(table)
    }
}

/// Effective color of each column of the table.
#[derive(Clone, Copy, Debug)]
struct AccountColors {
    name: Color,
    backends: Color,
    default: Color,
}

/// Loads the configuration, pointing at the wizard when none exists.
fn load_config(paths: &[PathBuf]) -> Result<Config> {
    match Config::from_paths_or_default(paths)? {
        Some(config) => Ok(config),
        None => anyhow::bail!(
            "No configuration found at {}, run `cardamum configure` to generate one",
            Config::target_path(paths)?.display(),
        ),
    }
}

/// One account, as rendered by [`AccountsTable`].
#[derive(Clone, Debug, Serialize)]
pub struct AccountRow {
    pub name: String,
    pub default: bool,
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
        #[cfg(feature = "people")]
        if account.people.is_some() {
            backends.push("people");
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

/// Table of accounts, and the JSON shape the command prints.
#[derive(Clone, Debug, Serialize)]
pub struct AccountsTable {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub arrangement: ContentArrangement,
    #[serde(skip)]
    colors: AccountColors,
    pub accounts: Vec<AccountRow>,
}

impl fmt::Display for AccountsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from(vec![
                Cell::new("NAME"),
                Cell::new("BACKENDS"),
                Cell::new("DEFAULT"),
            ]))
            .add_rows(self.accounts.iter().map(|account| {
                let mut row = Row::new();
                row.max_height(1);
                row.add_cell(Cell::new(&account.name).fg(self.colors.name));
                row.add_cell(Cell::new(account.backends.join(", ")).fg(self.colors.backends));
                row.add_cell(
                    Cell::new(if account.default { "yes" } else { "" }).fg(self.colors.default),
                );
                row
            }));

        writeln!(f)?;
        writeln!(f, "{table}")
    }
}
