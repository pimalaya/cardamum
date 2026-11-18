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
/// This is where it shows: the row handle to cancel it by, when it was queued,
/// the link id `card add` reported and its full name.
///
/// Staged updates and deletions need no such view, addressing cards that
/// already exist. A row of any kind the sync engine parked, refusing to
/// apply it without an operator, is listed `parked` with why: cancel it,
/// or fix what parked it. One the engine sent without learning its outcome
/// is listed `unconfirmed` instead: it may have taken effect.
///
/// JSON output: `{"cards": [<row>], "parked": [<row>]}`, each row the
/// family's `{"row", "kind", "collection", "linkId", "itemId", "producer",
/// "queuedAt", "attempts", "unconfirmed", "error"}` plus `handle` and
/// `fullName`.
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

/// One row of the store's queue, pending or parked: the family's queue row
/// (pimalaya-cli's output.md, "A queue row keeps one shape") plus the full
/// name of the card it concerns.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueRow {
    /// The queue row id, which `pimdir queue show` and `pimdir queue
    /// cancel` take. It names an action, not a card: a card created has no
    /// id until the sync engine applies the action.
    #[serde(rename = "row")]
    pub id: i64,
    /// The row's handle, its id under the store's identity (pimdir STORAGE
    /// §4.5), which `pimdir queue show` and `cancel` take and check, a
    /// restored copy of the store refusing it rather than another row.
    pub handle: String,
    /// The action kind as the store holds it (pimdir STORAGE §15.3):
    /// `add`, `update`, `remove`, ..., an intent or an application's own.
    pub kind: String,
    /// The address book the row is queued on.
    pub collection: String,
    /// The link id of the card the row creates, the `id` `card add`
    /// reported, `null` when the row creates none.
    pub link_id: Option<String>,
    /// The id of the stored card the row addresses, as `card list` prints
    /// it, `null` when it addresses none, a creation among them.
    pub item_id: Option<String>,
    /// The process that staged it.
    pub producer: String,
    /// When the row was appended, stamped by the store's own clock.
    pub queued_at: String,
    /// How many times the sync engine tried it.
    pub attempts: u64,
    /// Whether the row's outcome is unknown: the engine sent its request
    /// and cannot tell whether it took effect, so it may have.
    pub unconfirmed: bool,
    /// Why the row was parked, as the store records it, `null` while it is
    /// pending: an unconfirmed row's starts with `unconfirmed: `.
    pub error: Option<String>,
    /// The card's full name, from the body the row pins or the card it
    /// addresses, `null` when there is none to read.
    pub full_name: Option<String>,
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
    /// The cards staged for creation, pending.
    pub cards: Vec<PimdirQueueRow>,
    /// The parked rows, any kind.
    pub parked: Vec<PimdirQueueRow>,
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
                sanitize::text(&queued.handle).into_owned(),
                String::from("queued"),
                sanitize::text(&queued.queued_at).into_owned(),
                sanitize::text(&queued.kind).into_owned(),
                sanitize::text(&queued.collection).into_owned(),
                sanitize::text(queued.full_name.as_deref().unwrap_or_default()).into_owned(),
            ]);
        }

        for parked in &self.parked {
            listing.items.push([
                sanitize::text(&parked.handle).into_owned(),
                String::from(match parked.unconfirmed {
                    true => "unconfirmed",
                    false => "parked",
                }),
                sanitize::text(&parked.queued_at).into_owned(),
                sanitize::text(&parked.kind).into_owned(),
                sanitize::text(&parked.collection).into_owned(),
                sanitize::text(parked.error.as_deref().unwrap_or_default()).into_owned(),
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

    const STORE: &str = "e2646df031cd876334410c4df14ebd86";

    /// A pending and a parked row print one shape, the family's queue row
    /// with every key, `null` or not, and the text names the cancelling
    /// command.
    #[test]
    fn a_listing_keys_its_rows_as_the_family_does() {
        let output = PimdirQueueListOutput {
            account: String::from("perso"),
            addressbook: None,
            cards: vec![PimdirQueueRow {
                id: 3,
                handle: format!("{STORE}.1.3"),
                kind: String::from("add"),
                collection: String::from("book"),
                link_id: Some(String::from("urn:uuid:1")),
                item_id: None,
                producer: String::from("cardamum"),
                queued_at: String::from("2026-11-15T10:00:00.000Z"),
                attempts: 0,
                unconfirmed: false,
                error: None,
                full_name: Some(String::from("Jane Doe")),
            }],
            parked: vec![PimdirQueueRow {
                id: 4,
                handle: format!("{STORE}.1.4"),
                kind: String::from("update"),
                collection: String::from("book"),
                link_id: None,
                item_id: Some(String::from("7")),
                producer: String::from("cardamum"),
                queued_at: String::from("2026-11-15T10:01:00.000Z"),
                attempts: 1,
                unconfirmed: false,
                error: Some(String::from("conflict")),
                full_name: None,
            }],
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "cards": [{
                    "row": 3, "handle": format!("{STORE}.1.3"), "kind": "add", "collection": "book",
                    "linkId": "urn:uuid:1", "itemId": null,
                    "producer": "cardamum", "queuedAt": "2026-11-15T10:00:00.000Z",
                    "attempts": 0, "unconfirmed": false, "error": null,
                    "fullName": "Jane Doe",
                }],
                "parked": [{
                    "row": 4, "handle": format!("{STORE}.1.4"), "kind": "update", "collection": "book",
                    "linkId": null, "itemId": "7",
                    "producer": "cardamum", "queuedAt": "2026-11-15T10:01:00.000Z",
                    "attempts": 1, "unconfirmed": false, "error": "conflict",
                    "fullName": null,
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

        let rows = |handle: &str, state: &str| {
            text.lines()
                .any(|line| line.split_whitespace().take(2).eq([handle, state]))
        };
        assert!(rows(&format!("{STORE}.1.3"), "queued"), "{text}");
        assert!(rows(&format!("{STORE}.1.4"), "parked"), "{text}");
    }
}
