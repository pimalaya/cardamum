use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::contact_card::changes::JmapContactCardChangesOptions;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, render::ChangesReport};

/// The ContactCards changed since a state (`ContactCard/changes`).
///
/// JSON output: `{"created", "updated", "destroyed", "new_state",
/// "has_more_changes"}`.
#[derive(Debug, Parser)]
pub struct JmapContactCardChangesCommand {
    /// State token from a previous `get` / `query` (its `state`) or
    /// `changes`.
    #[arg(value_name = "SINCE-STATE")]
    pub since_state: String,
}

impl JmapContactCardChangesCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let out = client
            .contact_card_changes(self.since_state, JmapContactCardChangesOptions::default())?;

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
