//! # Pimdir status
//!
//! The `pimdir status` command and the report it renders.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::pimdir::backend::PimdirBackend;

/// Report what the local pimdir store holds.
///
/// Shows the account this client reads, every account the store groups
/// collections under, one row per address book with how many of its
/// cards carry a local body and how many creations are still queued, and
/// every write the store's owner parked in those address books.
///
/// An address book whose cards list but do not read is waiting on a sync
/// to hydrate them, and this is where to see that coming. A parked write
/// never lands: an update made from a card a sync changed since, a create
/// the server refused. It is shown by its queue row (ROW), the id the
/// store's own operator tooling retries or cancels it by.
///
/// JSON output: `{"account", "accounts", "addressbooks": [{"id", "name",
/// "total", "hydrated", "queued"}], "parked": [{"row", "addressbookId",
/// "action", "error"}]}`.
#[derive(Debug, Parser)]
pub struct PimdirStatusCommand;

impl PimdirStatusCommand {
    pub fn execute(self, printer: &mut impl Printer, mut backend: PimdirBackend) -> Result<()> {
        printer.out(backend.status()?)
    }
}

/// What a store holds, as `pimdir status` reports it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirStatusOutput {
    /// The account this client reads, `None` in a store grouping none.
    pub account: Option<String>,
    /// Every account the store groups collections under.
    pub accounts: Vec<String>,
    /// One row per address book of the account read.
    pub addressbooks: Vec<PimdirAddressbookStatus>,
    /// Every action the store's owner parked in those address books, in
    /// the order it was queued.
    pub parked: Vec<PimdirParkedStatus>,
}

/// One address book's row in a [`PimdirStatusOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirAddressbookStatus {
    /// The collection id.
    pub id: String,
    /// The collection display name, its id when it has none.
    pub name: String,
    /// How many live cards it holds.
    pub total: usize,
    /// How many of those carry a local body.
    pub hydrated: usize,
    /// How many creations are queued for it.
    ///
    /// They carry no public id until the store's owner applies them, so
    /// they list nowhere else.
    pub queued: usize,
}

/// One parked action in a [`PimdirStatusOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirParkedStatus {
    /// The queue row, which addresses the action rather than a card.
    pub row: i64,
    /// The address book the action targets.
    pub addressbook_id: String,
    /// The action kind (`add`, `update`, `remove`…).
    pub action: String,
    /// Why the owner parked it.
    pub error: String,
}

impl fmt::Display for PimdirStatusOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;

        match &self.account {
            Some(account) => writeln!(f, "Reading account: {account}")?,
            None => writeln!(f, "Reading account: none, the store groups no account")?,
        }

        match self.accounts.as_slice() {
            [] => writeln!(f, "Grouped accounts: none")?,
            accounts => writeln!(f, "Grouped accounts: {}", accounts.join(", "))?,
        }

        writeln!(f)?;

        for addressbook in &self.addressbooks {
            write!(
                f,
                "  {} ({}): {}/{} cards downloaded",
                addressbook.name, addressbook.id, addressbook.hydrated, addressbook.total
            )?;

            match addressbook.queued {
                0 => writeln!(f)?,
                queued => writeln!(f, ", {queued} queued for the next sync")?,
            }
        }

        if self.parked.is_empty() {
            return Ok(());
        }

        writeln!(f)?;
        writeln!(f, "Parked, never applied:")?;

        for parked in &self.parked {
            writeln!(
                f,
                "  ROW {} {} in {}: {}",
                parked.row, parked.action, parked.addressbook_id, parked.error
            )?;
        }

        Ok(())
    }
}
