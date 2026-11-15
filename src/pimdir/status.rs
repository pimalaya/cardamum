//! # Pimdir status
//!
//! The `pimdir status` command and the report it renders.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{columns::Columns, fields::Fields, printer::Printer, sanitize};
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
/// the server refused. It is shown by its queue row (ROW), the id
/// `pimdir queue cancel` takes; `pimdir queue list` lists the queued
/// creations themselves.
///
/// JSON output: `{"account", "accounts", "addressbooks": [{"id", "name",
/// "total", "hydrated", "queued"}], "parked": [{"row", "addressbookId",
/// "kind", "error"}]}`.
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
    pub kind: String,
    /// Why the owner parked it.
    pub error: String,
}

impl fmt::Display for PimdirStatusOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let account = match &self.account {
            Some(account) => sanitize::text(account).into_owned(),
            None => String::from("none, the store groups no account"),
        };
        let accounts = match self.accounts.as_slice() {
            [] => String::from("none"),
            accounts => sanitize::text(&accounts.join(", ")).into_owned(),
        };

        let mut fields = Fields::default();
        fields.push("Reading account", account);
        fields.push("Grouped accounts", accounts);
        write!(f, "{fields}")?;

        if !self.addressbooks.is_empty() {
            let header = ["ID", "CARDS", "DOWNLOADED", "QUEUED", "NAME"];
            let mut columns = Columns::new(header);

            for addressbook in &self.addressbooks {
                columns.push([
                    sanitize::text(&addressbook.id).into_owned(),
                    addressbook.total.to_string(),
                    addressbook.hydrated.to_string(),
                    addressbook.queued.to_string(),
                    sanitize::text(&addressbook.name).into_owned(),
                ]);
            }

            writeln!(f)?;
            write!(f, "{columns}")?;
        }

        if !self.parked.is_empty() {
            let mut columns = Columns::new(["ROW", "KIND", "ADDRESSBOOK", "ERROR"]);

            for parked in &self.parked {
                columns.push([
                    parked.row.to_string(),
                    sanitize::text(&parked.kind).into_owned(),
                    sanitize::text(&parked.addressbook_id).into_owned(),
                    sanitize::text(&parked.error).into_owned(),
                ]);
            }

            writeln!(f)?;
            write!(f, "{columns}")?;
            writeln!(f, "Parked by the store's owner, never applied")?;
        }

        Ok(())
    }
}
