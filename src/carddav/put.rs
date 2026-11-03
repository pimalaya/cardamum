//! # PUT command
//!
//! Creates or replaces the raw vCard bytes of a card with a WebDAV
//! `PUT`.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{carddav::client::CarddavClient, shared::card::vcard::read_source};

/// PUT a card: create or replace its raw vCard bytes.
///
/// A plain PUT creates or overwrites unconditionally. `--if-none-match
/// '*'` makes it a create-only PUT, failing when the resource exists,
/// and `--if-match <ETAG>` gates the replace on the resource being
/// unchanged (RFC 9110).
///
/// JSON output: `{"id", "etag"}`.
#[derive(Debug, Parser)]
pub struct CarddavPutCommand {
    /// Identifier of the parent addressbook.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Card resource id (its href last path segment).
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
    /// vCard written to the card resource: a path to a file, raw vCard
    /// contents, or `-` for stdin.
    #[arg(value_name = "VCARD")]
    pub vcard: String,
    /// Gate the replace on the resource being unchanged (If-Match).
    #[arg(long, value_name = "ETAG", conflicts_with = "if_none_match")]
    pub if_match: Option<String>,
    /// Create-only PUT: fail if the resource already exists (pass `*`).
    #[arg(long, value_name = "VALUE", conflicts_with = "if_match")]
    pub if_none_match: Option<String>,
}

impl CarddavPutCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let contents = read_source(&self.vcard)?;

        let etag = if self.if_none_match.is_some() {
            client
                .create_card(&self.addressbook_id, &self.card_id, contents)?
                .etag
        } else {
            client
                .update_card(
                    &self.addressbook_id,
                    &self.card_id,
                    contents,
                    self.if_match.as_deref(),
                )?
                .etag
        };

        printer.out(CarddavPutOutput {
            id: self.card_id,
            etag,
        })
    }
}

/// The card resource `carddav put` wrote.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavPutOutput {
    /// The card resource id.
    pub id: String,
    /// The ETag the server answered, `null` when it sent none.
    pub etag: Option<String>,
}

impl fmt::Display for CarddavPutOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let etag = self.etag.as_deref().unwrap_or("(none)");
        writeln!(f, "Card `{}` successfully put (etag {etag})", self.id)
    }
}
