//! # Repl command
//!
//! The `repl` subcommand, a session over stdin and stdout running the
//! CLI's own commands against one account, so the unified client connects
//! once rather than once per command.
//!
//! pimalaya-cli's [`Repl`] runs the loop: each line split as a shell splits
//! it, parsed with the one-shot grammar plus `--json` and run over the
//! [`Session`] kept here, every answer on stdout, the session flags refused
//! by name, a nested `repl` or `mcp` line refused coded `usage-invalid`
//! before it reaches this crate, and a `--version` line answered with the
//! version [`Command`] carries. It claims stdin, so a command that would
//! read its own input there is refused
//! (`pimalaya_cli::session::check_stdin`), and so is the composer, whose
//! editor would fight the session for the terminal.

use anyhow::{Result, anyhow};
use clap::Parser;
use pimalaya_cli::{printer::Printer, repl::Repl};

use crate::{
    cli::{Command, resolve_account},
    error::ErrorCode,
    session::Session,
    shared::card::cli::CardCommand,
};

/// Start a session running commands over one account.
///
/// Each line is a command without the binary name (`addressbook list`,
/// `card read -k personal 42`), the unified commands running over one
/// client connected once for the whole session; the account and
/// `--backend` are the session's, and `--json` on a line switches that
/// line alone to JSON. A command reading stdin and the composer (`-i`)
/// are refused. `quit`, `exit`, or EOF ends it.
#[derive(Debug, Parser)]
pub struct ReplCommand;

impl ReplCommand {
    /// Runs the read-eval-print loop until EOF or a quit command, keeping
    /// the session's client across lines, each line printing as
    /// `printer` does unless it asks for JSON.
    pub fn execute(self, printer: &impl Printer, session: &mut Session) -> Result<()> {
        let account = session.account;

        // NOTE: resolved once up front, so a missing configuration or
        // account fails the session rather than its first command.
        resolve_account(account.config_paths, account.name)?;

        let mut repl = Repl::new("cardamum", printer.is_json());
        repl.fixed.extend(["-b", "--backend"]);

        repl.run::<Command, ErrorCode>(|printer, cmd| match cmd {
            Command::Card(CardCommand::Build(cmd)) if cmd.composer.interactive => Err(composer()),
            Command::Card(CardCommand::Add(cmd)) if cmd.composer.interactive => Err(composer()),
            Command::Card(CardCommand::Update(cmd)) if cmd.composer.interactive => Err(composer()),
            cmd => cmd.execute(printer, session),
        })
    }
}

/// The refusal of the composer inside a session: its editor would take
/// the terminal the session reads its commands from, and pimalaya-cli
/// installs no Ctrl-C handler, so an interrupted editor would end the
/// session with it.
fn composer() -> anyhow::Error {
    anyhow!(
        "Cannot run the composer (-i) inside a session; pass the vCard or the field flags instead"
    )
}
