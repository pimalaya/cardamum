//! # Multiget report command
//!
//! Runs the RFC 6352 `addressbook-multiget` REPORT and renders the
//! fetched cards.

use anyhow::Result;
use clap::Parser;
use io_webdav::rfc4918::typed::WebdavUnreadableEntry;
use pimalaya_cli::printer::Printer;

use crate::{
    carddav::{
        backend::unreadable_row,
        client::CarddavClient,
        report::entries::{CarddavCardEntriesOutput, EntryRow},
    },
    error::{ErrorCode, ItemError, partial_when},
};

/// Fetch the named cards in one round-trip (RFC 6352 §8.7).
///
/// The `addressbook-multiget` REPORT: one request brings back the body
/// and ETag of every id given.
///
/// JSON output: `{"cards": [{"id", "etag", "contents", "encoding"}],
/// "truncated", "errors": [{"id", "code", "error", "sources"}]}`, each
/// vCard as text (`encoding` `utf-8`) or as the base64 of bytes that are
/// not UTF-8 (`encoding` `base64`); `errors` holds the error of each
/// card the server answered without its data, `card-not-found` for an id
/// it does not hold, `[]` when none, the command then exiting 3.
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
        let ids: Vec<&str> = self.card_ids.iter().map(String::as_str).collect();
        let ok = client.multiget_cards(&self.addressbook_id, &ids)?;

        let output = CarddavCardEntriesOutput {
            rows: ok.cards.into_iter().map(EntryRow::from).collect(),
            truncated: false,
            errors: ok.unreadable.into_iter().map(multiget_row).collect(),
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// The entry of a card the multiget named and the server answered
/// without its data: a row answered 404 or 410 is `card-not-found`, as a
/// card request is, any other reads as a listing row.
fn multiget_row(row: WebdavUnreadableEntry) -> ItemError {
    match row.status {
        Some(status @ (404 | 410)) => ItemError {
            id: Some(row.id),
            code: Some(ErrorCode::CardNotFound),
            error: format!("The server holds no such card, status {status}"),
            sources: row.failures.iter().map(ToString::to_string).collect(),
        },
        _ => unreadable_row(row),
    }
}
