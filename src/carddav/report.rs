use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Row, Table};
use io_webdav::rfc6352::card::CardEntry;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::carddav::client::CarddavClient;

/// Run an `addressbook-query` REPORT against the given addressbook.
///
/// Lists every card UID + ETag advertised by the server, without
/// fetching the bodies. Wraps RFC 6352 §8.6.
#[derive(Debug, Parser)]
pub struct CarddavReportCommand {
    /// Identifier of the addressbook to query.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
}

impl CarddavReportCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let entries = client.list_cards(&self.addressbook_id)?;

        let table = ReportTable {
            preset: client.account.table_preset().to_string(),
            rows: entries.into_iter().map(EntryRow::from).collect(),
        };

        printer.out(table)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ReportTable {
    #[serde(skip)]
    pub preset: String,
    #[serde(rename = "entries")]
    pub rows: Vec<EntryRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EntryRow {
    pub id: String,
    pub etag: Option<String>,
}

impl From<CardEntry> for EntryRow {
    fn from(entry: CardEntry) -> Self {
        Self {
            id: entry.id,
            etag: entry.etag,
        }
    }
}

impl fmt::Display for ReportTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_header(Row::from([Cell::new("ID"), Cell::new("ETAG")]))
            .add_rows(self.rows.iter().map(|e| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&e.id))
                    .add_cell(Cell::new(e.etag.as_deref().unwrap_or("")));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}
