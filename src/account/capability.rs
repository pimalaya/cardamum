//! # Account capability command
//!
//! Lists what the backend an account resolves to serves of the unified
//! API, from the capability tables, without connecting.

use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{columns::Columns, printer::Printer};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    backend::Backend, capability::Capability, cli::resolve_account,
    shared::client::AddressbookClient,
};

/// List what the account serves of the unified API.
///
/// Resolves the backend the unified commands run against (the first
/// configured one `--backend` allows) and prints, for each capability,
/// its level (`full`, `partial`, `server` or `none`) and what the level
/// leaves out. Nothing is reached: this reads the configuration only.
///
/// JSON output: `{"account", "backend", "capabilities": [{"name",
/// "level", "detail"}]}`.
#[derive(Debug, Parser)]
pub struct AccountCapabilityCommand;

impl AccountCapabilityCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        let (config, name, account_config) = resolve_account(config_paths, account_name)?;
        let client = AddressbookClient::new(config, name, account_config, backend)?;
        let backend = client.backend().to_string();

        let capabilities = Capability::ALL
            .into_iter()
            .map(|capability| {
                let support = crate::capability::support(&backend, capability);
                AccountCapability {
                    name: capability.name(),
                    level: support.level(),
                    detail: support.detail(),
                }
            })
            .collect();

        printer.out(AccountCapabilityOutput {
            account: client.name,
            backend,
            capabilities,
        })
    }
}

/// The `account capability` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountCapabilityOutput {
    /// The name of the account.
    pub account: String,
    /// The backend the unified commands resolve to.
    pub backend: String,
    /// Each capability of the unified API, in a fixed order.
    pub capabilities: Vec<AccountCapability>,
}

/// One capability, as the resolved backend serves it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountCapability {
    /// The capability's name, pimdir STORAGE Annex B's without the domain
    /// word, as `card.update.if-match`.
    pub name: &'static str,
    /// `full`, `partial`, `server` or `none`.
    pub level: &'static str,
    /// What the level leaves out, `null` for a full one.
    pub detail: Option<&'static str>,
}

impl fmt::Display for AccountCapabilityOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::new(["CAPABILITY", "BACKEND", "LEVEL", "DETAIL"]);

        for capability in &self.capabilities {
            columns.push([
                capability.name,
                self.backend.as_str(),
                capability.level,
                capability.detail.unwrap_or_default(),
            ]);
        }

        write!(f, "{columns}")
    }
}
