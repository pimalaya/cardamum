//! # Multiget report command
//!
//! Runs the RFC 6352 `addressbook-multiget` REPORT and renders the
//! fetched cards.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::carddav::{
    client::CarddavClient,
    report::entries::{CarddavCardEntriesOutput, EntryRow},
};

/// Fetch the named cards in one round-trip (RFC 6352 §8.7).
///
/// The `addressbook-multiget` REPORT: one request brings back the body
/// and ETag of every id given.
///
/// JSON output: `{"cards": [{"id", "etag", "contents"}]}`.
#[derive(Debug, Parser)]
pub struct CarddavReportMultigetCommand {
    /// Identifier of the addressbook to query.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Card resource ids to fetch.
    #[arg(value_name = "CARD-ID", required = true)]
    pub card_ids: Vec<String>,
}

impl CarddavReportMultigetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();
        let ids: Vec<&str> = self.card_ids.iter().map(String::as_str).collect();
        let entries = client.multiget_cards(&self.addressbook_id, &ids)?;

        printer.out(CarddavCardEntriesOutput {
            preset,
            id_color,
            rows: entries.into_iter().map(EntryRow::from).collect(),
        })
    }
}
