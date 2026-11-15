//! # Card commands
//!
//! Dispatches the unified API card subcommands over the session's unified
//! client, `build` reaching none.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::{
    capability,
    session::Session,
    shared::card::{
        add::CardAddCommand, build::CardBuildCommand, delete::CardDeleteCommand,
        list::CardListCommand, read::CardReadCommand, update::CardUpdateCommand,
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
    #[command(aliases = ["create", "new"], after_long_help = capability::help("card", "add"))]
    Add(Box<CardAddCommand>),
    #[command(after_long_help = capability::help("card", "update"))]
    Update(Box<CardUpdateCommand>),
    #[command(alias = "rm", after_long_help = capability::help("card", "delete"))]
    Delete(CardDeleteCommand),
}

impl CardCommand {
    /// Runs the subcommand, over the session's unified client but for
    /// `build`, which reaches no backend.
    pub fn execute(self, printer: &mut impl Printer, session: &mut Session) -> Result<()> {
        let account = session.account;

        match self {
            // NOTE: `build` reaches no backend, so it resolves no account
            // either: printing a vCard on a machine holding no
            // configuration is what it is for.
            Self::Build(cmd) => cmd.execute(printer, account.config_paths, account.name),
            Self::List(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Read(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Add(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Update(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
            Self::Delete(cmd) => session.run_unified(printer, |p, client| cmd.execute(p, client)),
        }
    }
}
