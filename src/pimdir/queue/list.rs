//! # pimdir queue list
//!
//! The `pimdir queue list` command, listing the card creations staged and
//! the rows the store's owner parked in the account's address books.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{listing::Listing, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{backend::Backend, pimdir::backend::PimdirBackend};

/// List the cards staged for creation and the parked rows.
///
/// A card added on pimdir waits in the store's queue until the sync engine
/// applies it, and has no id until then, so `card list` cannot show it.
/// This is where it shows: the row id to cancel it by, when it was queued,
/// the link id `card add` reported and its full name.
///
/// Staged updates and deletions need no such view, addressing cards that
/// already exist. A row of any kind the sync engine parked, refusing to
/// apply it without an operator, is listed `parked` with why: cancel it,
/// or fix what parked it. One the engine sent without learning its outcome
/// is listed `unconfirmed` instead: it may have taken effect.
///
/// JSON output: `{"cards": [{"row", "queuedAt", "producer", "collection",
/// "kind", "linkId", "fullName"}], "parked": [{"row", "queuedAt", "producer",
/// "collection", "kind", "attempts", "unconfirmed", "error"}]}`.
#[derive(Debug, Parser)]
pub struct PimdirQueueListCommand {
    /// Address book to list, every one of the account's when omitted.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub addressbook: Option<String>,
}

impl PimdirQueueListCommand {
    /// Lists the creations and the parked rows, and prints them.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        account: &str,
        backend: &PimdirBackend,
    ) -> Result<()> {
        let addressbook = self.addressbook.as_deref();
        let cards = backend.queued_cards(addressbook)?;
        let parked = backend.parked_rows(addressbook)?;

        printer.out(PimdirQueueListOutput {
            account: account.to_owned(),
            addressbook: self.addressbook,
            cards,
            parked,
        })
    }
}

/// One card creation waiting in the store's queue.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueuedCard {
    /// The queue row id, which `pimdir queue cancel` takes. It names a
    /// pending action, not a card: the card has no id until the sync
    /// engine applies the action.
    #[serde(rename = "row")]
    pub id: i64,
    /// When the row was appended, stamped by the store's own clock.
    pub queued_at: String,
    /// The process that staged it.
    pub producer: String,
    /// The address book the card is staged in.
    pub collection: String,
    /// The action kind, `add`: the one row a card has no id for.
    pub kind: String,
    /// The link id `card add` reported, derived from the card's `UID`.
    pub link_id: Option<String>,
    /// The card's full name, from the body the row pins, `null` when that
    /// body is not in the store.
    pub full_name: Option<String>,
}

/// One row the sync engine parked: it will not apply it again without an
/// operator.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirParkedRow {
    /// The queue row id, which `pimdir queue cancel` takes.
    #[serde(rename = "row")]
    pub id: i64,
    /// When the row was appended, stamped by the store's own clock.
    pub queued_at: String,
    /// The process that staged it.
    pub producer: String,
    /// The address book the row targets.
    pub collection: String,
    /// The action kind: `add`, `update`, `remove`, ...
    pub kind: String,
    /// How many times the engine tried it before parking it.
    pub attempts: i64,
    /// Whether the row is unconfirmed: the engine sent its request and
    /// cannot tell whether it took effect, so it may have.
    pub unconfirmed: bool,
    /// Why it was parked, as the store records it: an unconfirmed row's
    /// starts with `unconfirmed: `.
    pub error: String,
}

/// The `pimdir queue list` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueListOutput {
    /// The name of the account listed.
    #[serde(skip)]
    pub account: String,
    /// The address book listed, `None` for every one of the account's.
    #[serde(skip)]
    pub addressbook: Option<String>,
    /// The cards staged for creation.
    pub cards: Vec<PimdirQueuedCard>,
    /// The parked rows, any kind.
    pub parked: Vec<PimdirParkedRow>,
}

impl fmt::Display for PimdirQueueListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let backend = Backend::Pimdir.to_string();
        let mut listing = Listing::new(&self.account, Some(&backend));
        let addressbook = self.addressbook.as_deref().unwrap_or("all");
        listing.head("Addressbook", addressbook);
        listing.empty = Some("Nothing queued nor parked");

        for queued in &self.cards {
            listing.items.push([
                queued.id.to_string(),
                String::from("queued"),
                sanitize::text(&queued.queued_at).into_owned(),
                sanitize::text(&queued.kind).into_owned(),
                sanitize::text(&queued.collection).into_owned(),
                sanitize::text(queued.full_name.as_deref().unwrap_or_default()).into_owned(),
            ]);
        }

        for parked in &self.parked {
            listing.items.push([
                parked.id.to_string(),
                String::from(match parked.unconfirmed {
                    true => "unconfirmed",
                    false => "parked",
                }),
                sanitize::text(&parked.queued_at).into_owned(),
                sanitize::text(&parked.kind).into_owned(),
                sanitize::text(&parked.collection).into_owned(),
                sanitize::text(&parked.error).into_owned(),
            ]);
        }

        if !listing.items.rows.is_empty() {
            let cancel = String::from("cardamum pimdir queue cancel <ROW>");
            listing.header.push(("Cancel", cancel));
        }

        write!(f, "{listing}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The JSON keys are Himalaya's `pimdir queue list` ones, `row` and
    /// `kind` among them, and the text names the cancelling command.
    #[test]
    fn a_listing_keys_its_rows_as_himalaya_does() {
        let output = PimdirQueueListOutput {
            account: String::from("perso"),
            addressbook: None,
            cards: vec![PimdirQueuedCard {
                id: 3,
                queued_at: String::from("2026-11-15T10:00:00Z"),
                producer: String::from("cardamum"),
                collection: String::from("book"),
                kind: String::from("add"),
                link_id: Some(String::from("urn:uuid:1")),
                full_name: Some(String::from("Jane Doe")),
            }],
            parked: vec![PimdirParkedRow {
                id: 4,
                queued_at: String::from("2026-11-15T10:01:00Z"),
                producer: String::from("cardamum"),
                collection: String::from("book"),
                kind: String::from("update"),
                attempts: 1,
                unconfirmed: false,
                error: String::from("conflict"),
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "cards": [{
                    "row": 3, "queuedAt": "2026-11-15T10:00:00Z",
                    "producer": "cardamum", "collection": "book",
                    "kind": "add", "linkId": "urn:uuid:1", "fullName": "Jane Doe",
                }],
                "parked": [{
                    "row": 4, "queuedAt": "2026-11-15T10:01:00Z",
                    "producer": "cardamum", "collection": "book",
                    "kind": "update", "attempts": 1, "unconfirmed": false,
                    "error": "conflict",
                }],
            })
        );

        let text = output.to_string();
        assert!(text.contains("Addressbook:"), "{text}");
        assert!(
            text.contains("cardamum pimdir queue cancel <ROW>"),
            "{text}"
        );
        assert!(text.contains("Jane Doe"), "{text}");
    }
}
