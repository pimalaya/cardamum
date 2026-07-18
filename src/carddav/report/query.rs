use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::carddav::{
    client::CarddavClient,
    report::entries::{CardEntriesReport, EntryRow},
};

/// `addressbook-query` REPORT: the cards matching a match-all filter,
/// with their ETags and bodies (RFC 6352 §8.6).
///
/// JSON output: `{"cards": [{"id", "etag", "contents"}]}`.
#[derive(Debug, Parser)]
pub struct CarddavReportQueryCommand {
    /// Identifier of the addressbook to query.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
}

impl CarddavReportQueryCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();
        let entries = client.list_cards(&self.addressbook_id)?;

        printer.out(CardEntriesReport {
            preset,
            id_color,
            rows: entries.into_iter().map(EntryRow::from).collect(),
        })
    }
}
