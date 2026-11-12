//! # Card commands
//!
//! Dispatches the unified API card subcommands, resolving the account
//! each one runs against, or none at all.

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    backend::Backend,
    capability,
    cli::resolve_account,
    shared::{
        card::{
            build::CardBuildCommand, create::CardCreateCommand, delete::CardDeleteCommand,
            list::CardListCommand, read::CardReadCommand, update::CardUpdateCommand,
        },
        client::AddressbookClient,
    },
};

/// Manage vCards using the unified API.
///
/// Runs against the first backend defined for the default account, or for
/// the account named by `--account`.
#[derive(Debug, Subcommand)]
pub enum CardCommand {
    #[command(visible_alias = "ls", after_long_help = capability::help("card", "list"))]
    List(CardListCommand),
    #[command(after_long_help = capability::help("card", "read"))]
    Read(CardReadCommand),
    Build(Box<CardBuildCommand>),
    #[command(visible_alias = "new", after_long_help = capability::help("card", "create"))]
    Create(Box<CardCreateCommand>),
    #[command(after_long_help = capability::help("card", "update"))]
    Update(Box<CardUpdateCommand>),
    #[command(visible_alias = "rm", after_long_help = capability::help("card", "delete"))]
    Delete(CardDeleteCommand),
}

impl CardCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<()> {
        match self {
            // NOTE: `build` reaches no backend, so it resolves no account
            // either: printing a vCard on a machine holding no
            // configuration is what it is for.
            Self::Build(cmd) => cmd.execute(printer, config_paths, account_name),
            Self::List(cmd) => {
                let client = client(config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Read(cmd) => {
                let client = client(config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Create(cmd) => {
                let client = client(config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Update(cmd) => {
                let client = client(config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
            Self::Delete(cmd) => {
                let client = client(config_paths, account_name, backend)?;
                cmd.execute(printer, client)
            }
        }
    }
}

/// Resolves the account a card command runs against, and opens a client
/// on the backend serving it.
fn client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
    backend: Backend,
) -> Result<AddressbookClient> {
    let (config, name, account_config) = resolve_account(config_paths, account_name)?;

    AddressbookClient::new(config, name, account_config, backend)
}
