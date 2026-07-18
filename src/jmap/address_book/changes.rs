use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::address_book::changes::JmapAddressBookChangesOptions;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, render::ChangesReport};

/// The AddressBooks changed since a state (`AddressBook/changes`).
///
/// JSON output: `{"created", "updated", "destroyed", "new_state",
/// "has_more_changes"}`.
#[derive(Debug, Parser)]
pub struct JmapAddressBookChangesCommand {
    /// State token from a previous `get` (its `state`) or `changes`.
    #[arg(value_name = "SINCE-STATE")]
    pub since_state: String,
}

impl JmapAddressBookChangesCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let out = client
            .address_book_changes(self.since_state, JmapAddressBookChangesOptions::default())?;

        printer.out(ChangesReport {
            preset,
            created: out.created,
            updated: out.updated,
            destroyed: out.destroyed,
            new_state: out.new_state,
            has_more_changes: out.has_more_changes,
        })
    }
}
