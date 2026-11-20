//! # pimdir queue show
//!
//! The `pimdir queue show` command, saying where one queue row stands.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_pimdir::{
    client::producer::{PimdirActionStatus, PimdirReceiptRefusal},
    codec::PimdirActionHandle,
};
use pimalaya_cli::{printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::pimdir::{
    backend::{PimdirBackend, uncancellable},
    queue::list::PimdirQueueRow,
};

/// Show where one queue row stands, by the row id `pimdir queue list`
/// printed, or by a row handle naming its store
/// (`<store-id>.<incarnation>.<id>`), refused `store-reset` when the store
/// is another one.
///
/// A row is `pending` until the sync engine applies it, `parked` when the
/// engine gave up on it (with why), `unconfirmed` when the engine sent its
/// request and cannot tell whether it took effect (it waits for the engine
/// to confirm it, or for a cancellation), and `applied` once done, with
/// the id of the card it acted on. An applied row lists the refusals
/// standing on that card: a server that refused for good the change the
/// row led to, with its reason, held until the card is edited or removed.
/// The store keeps what became of an applied row for seven days, and while
/// a refusal stands on its card, under the address book it was queued on,
/// or under none once that address book is gone. Past that, a cancelled
/// row and a row of another account are refused `row-not-found`.
///
/// JSON output: the `pimdir queue list` row, `{"row", "handle", "kind",
/// "collection", "linkId", "itemId", "producer", "queuedAt", "attempts",
/// "unconfirmed", "error", "fullName"}`, plus `"state"`, `"appliedAt"` and
/// `"refusals": [{"collection", "source", "reason", "refusedAt"}]`. An
/// applied row, which the queue holds no more, prints `null` for what only
/// the queue kept: `kind`, `producer`, `queuedAt` and `attempts`.
#[derive(Debug, Parser)]
pub struct PimdirQueueShowCommand {
    /// The queue row id, or its handle.
    #[arg(value_name = "ROW")]
    pub id: PimdirActionHandle,
}

impl PimdirQueueShowCommand {
    /// Looks the row up in the account's queue and receipts.
    pub fn execute(self, printer: &mut impl Printer, backend: &PimdirBackend) -> Result<()> {
        let handle = backend.named(&self.id)?;

        let output = match backend.queue_row(&self.id)? {
            PimdirActionStatus::Pending { collection, .. } => {
                let row = backend.listed_row(&handle, &collection)?;
                PimdirQueueShowOutput::listed(row, PimdirQueueState::Pending)
            }
            PimdirActionStatus::Parked { collection, .. } => {
                let row = backend.listed_row(&handle, &collection)?;
                PimdirQueueShowOutput::listed(row, PimdirQueueState::Parked)
            }
            PimdirActionStatus::Unconfirmed { collection, .. } => {
                let row = backend.listed_row(&handle, &collection)?;
                PimdirQueueShowOutput::listed(row, PimdirQueueState::Unconfirmed)
            }
            PimdirActionStatus::Applied {
                applied_at,
                collection,
                seq,
                refusals,
            } => {
                let full_name = match (&collection, seq) {
                    (Some(collection), Some(seq)) => backend.full_name(collection, seq)?,
                    _ => None,
                };

                PimdirQueueShowOutput::applied(
                    &handle, applied_at, collection, seq, refusals, full_name,
                )
            }
            status => return Err(uncancellable(&self.id, &status)),
        };

        printer.out(output)
    }
}

/// Where a queue row stands.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PimdirQueueState {
    /// Waiting for the sync engine.
    Pending,
    /// Given up on by the sync engine; `error` says why.
    Parked,
    /// Sent by the sync engine, which cannot tell whether it took effect:
    /// it may have, and waits for the engine to confirm it or for a
    /// cancellation; `error` holds what the engine recorded.
    Unconfirmed,
    /// Applied by the sync engine; `itemId` is the card it acted on, and
    /// `refusals` the servers that refused the change it led to.
    Applied,
}

/// The `pimdir queue show` output: the `pimdir queue list` row, plus what
/// only a read carries.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueShowOutput {
    /// The queue row id.
    pub row: i64,
    /// The row's handle, its id under the store's identity (pimdir STORAGE
    /// §4.5): the handle given, or the bare id given named under this
    /// store's identity.
    pub handle: String,
    /// The action kind as the store holds it, `null` once applied.
    pub kind: Option<String>,
    /// The address book the row is queued on, `null` for an applied row
    /// whose address book is gone since.
    pub collection: Option<String>,
    /// The link id of the card the row creates, `null` when it creates
    /// none or is applied.
    pub link_id: Option<String>,
    /// The id of the stored card the row addresses, or the one an applied
    /// row acted on (the one an add created), as `card read` takes it.
    pub item_id: Option<String>,
    /// The process that staged it, `null` once applied.
    pub producer: Option<String>,
    /// When the row was appended, `null` once applied.
    pub queued_at: Option<String>,
    /// How many times the sync engine tried it, `null` once applied.
    pub attempts: Option<u64>,
    /// Whether the row's outcome is unknown.
    pub unconfirmed: bool,
    /// Why the row was parked, or what the sync engine recorded of an
    /// unconfirmed one.
    pub error: Option<String>,
    /// The card's full name, `null` when there is none to read.
    pub full_name: Option<String>,
    /// Where it stands.
    pub state: PimdirQueueState,
    /// When the row was applied, RFC 3339.
    pub applied_at: Option<String>,
    /// The refusals standing on the card, empty but for an applied row.
    pub refusals: Vec<PimdirQueueRefusal>,
}

/// A refusal standing on the card an applied row acted on (pimdir
/// STORAGE §15.4): the server of `source` refused for good the change the
/// row led to, and the sync engine holds it until the card changes again.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueRefusal {
    /// The address book filing the card, the target of a move.
    pub collection: String,
    /// The source whose server refused the change.
    pub source: String,
    /// The text the server gave.
    pub reason: String,
    /// When the refusal was recorded, RFC 3339.
    pub refused_at: String,
}

impl From<PimdirReceiptRefusal> for PimdirQueueRefusal {
    fn from(refusal: PimdirReceiptRefusal) -> Self {
        Self {
            collection: refusal.collection,
            source: refusal.source,
            reason: refusal.reason,
            refused_at: refusal.refused_at,
        }
    }
}

impl PimdirQueueShowOutput {
    /// The listed `row`, standing at `state`.
    pub fn listed(row: PimdirQueueRow, state: PimdirQueueState) -> Self {
        Self {
            row: row.id,
            handle: row.handle,
            kind: Some(row.kind),
            collection: Some(row.collection),
            link_id: row.link_id,
            item_id: row.item_id,
            producer: Some(row.producer),
            queued_at: Some(row.queued_at),
            attempts: Some(row.attempts),
            unconfirmed: row.unconfirmed,
            error: row.error,
            full_name: row.full_name,
            state,
            applied_at: None,
            refusals: Vec::new(),
        }
    }

    /// The row `handle` names, applied at `applied_at` in `collection` on
    /// the card `seq` of full name `full_name`, `refusals` standing there.
    pub fn applied(
        handle: &PimdirActionHandle,
        applied_at: String,
        collection: Option<String>,
        seq: Option<i64>,
        refusals: Vec<PimdirReceiptRefusal>,
        full_name: Option<String>,
    ) -> Self {
        Self {
            row: handle.id,
            handle: handle.to_string(),
            kind: None,
            collection,
            link_id: None,
            item_id: seq.map(|seq| seq.to_string()),
            producer: None,
            queued_at: None,
            attempts: None,
            unconfirmed: false,
            error: None,
            full_name,
            state: PimdirQueueState::Applied,
            applied_at: Some(applied_at),
            refusals: refusals.into_iter().map(Into::into).collect(),
        }
    }
}

impl fmt::Display for PimdirQueueShowOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = &self.handle;
        let collection = sanitize::text(self.collection.as_deref().unwrap_or_default());
        let kind = sanitize::text(self.kind.as_deref().unwrap_or_default());

        match self.state {
            PimdirQueueState::Pending => {
                writeln!(f, "Row {id} ({kind} in {collection}) is pending")
            }
            PimdirQueueState::Parked => {
                let error = sanitize::text(self.error.as_deref().unwrap_or_default());
                writeln!(f, "Row {id} ({kind} in {collection}) is parked: {error}")
            }
            PimdirQueueState::Unconfirmed => {
                let detail = sanitize::text(self.error.as_deref().unwrap_or_default());
                writeln!(
                    f,
                    "Row {id} ({kind} in {collection}) is unconfirmed, sent and maybe \
                     performed: {detail}"
                )
            }
            PimdirQueueState::Applied => {
                match self.collection {
                    Some(_) => write!(f, "Row {id} was applied in {collection}")?,
                    None => write!(f, "Row {id} was applied in an address book deleted since")?,
                }
                if let Some(item_id) = &self.item_id {
                    write!(f, ", on card {}", sanitize::text(item_id))?;
                }
                writeln!(f)?;
                for refusal in &self.refusals {
                    writeln!(
                        f,
                        "Refused by {} in {} at {}: {}",
                        sanitize::text(&refusal.source),
                        sanitize::text(&refusal.collection),
                        sanitize::text(&refusal.refused_at),
                        sanitize::text(&refusal.reason),
                    )?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use io_pimdir::codec::PimdirStoreIdentity;

    use super::*;

    const STORE: &str = "e2646df031cd876334410c4df14ebd86";

    /// The handle of row `id` of the store `STORE`.
    fn handle(id: i64) -> PimdirActionHandle {
        PimdirActionHandle {
            id,
            store: Some(PimdirStoreIdentity {
                store_id: STORE.into(),
                incarnation: 1,
            }),
        }
    }

    /// An applied add prints the list row's keys, `null` where only the
    /// queue kept a value, the card it created as `itemId`.
    #[test]
    fn an_applied_add_prints_the_list_row_and_its_card() {
        let row = PimdirQueueShowOutput::applied(
            &handle(12),
            "2026-10-04T20:00:00Z".into(),
            Some("carddav/contacts".into()),
            Some(42),
            Vec::new(),
            Some("Jane Doe".into()),
        );
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "row": 12,
                "handle": format!("{STORE}.1.12"),
                "kind": null,
                "collection": "carddav/contacts",
                "linkId": null,
                "itemId": "42",
                "producer": null,
                "queuedAt": null,
                "attempts": null,
                "unconfirmed": false,
                "error": null,
                "fullName": "Jane Doe",
                "state": "applied",
                "appliedAt": "2026-10-04T20:00:00Z",
                "refusals": [],
            })
        );
    }

    /// A queued row prints the row `pimdir queue list` prints for it, every
    /// key of it, then its state.
    #[test]
    fn a_queued_row_prints_its_list_row() {
        let listed = PimdirQueueRow {
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
        };
        let mut expected = serde_json::to_value(&listed).unwrap();

        let row = PimdirQueueShowOutput::listed(listed, PimdirQueueState::Pending);
        let json = serde_json::to_value(&row).unwrap();

        let object = expected.as_object_mut().unwrap();
        object.insert("state".into(), "pending".into());
        object.insert("appliedAt".into(), serde_json::Value::Null);
        object.insert("refusals".into(), serde_json::json!([]));
        let listed: Vec<String> = object.keys().cloned().collect();
        assert_eq!(json, expected);

        let keys: Vec<String> = json.as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, listed);
        assert_eq!(
            row.to_string(),
            format!("Row {STORE}.1.3 (add in book) is pending\n")
        );
    }

    /// A row its performer sent and could not confirm is no failure and no
    /// success (pimdir STORAGE §15.2): it reads `unconfirmed`, with what
    /// the performer recorded, never `parked`.
    #[test]
    fn an_unconfirmed_row_says_it_may_have_taken_effect() {
        let listed = PimdirQueueRow {
            id: 9,
            handle: format!("{STORE}.1.9"),
            kind: String::from("add"),
            collection: String::from("book"),
            link_id: None,
            item_id: None,
            producer: String::from("cardamum"),
            queued_at: String::from("2026-11-15T10:00:00.000Z"),
            attempts: 1,
            unconfirmed: true,
            error: Some(String::from("unconfirmed: connection lost after PUT")),
            full_name: None,
        };
        let row = PimdirQueueShowOutput::listed(listed, PimdirQueueState::Unconfirmed);

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["state"], "unconfirmed");
        assert_eq!(json["unconfirmed"], true);

        let line = row.to_string();
        assert!(line.contains("maybe performed"), "{line}");
        assert!(line.contains("connection lost after PUT"), "{line}");
    }

    /// An applied row whose change a server refused for good shows applied,
    /// then a line per refusal with its source and reason (pimdir STORAGE
    /// §15.4), and the JSON lists them.
    #[test]
    fn an_applied_row_prints_the_refusals_on_its_card() {
        let row = PimdirQueueShowOutput::applied(
            &handle(12),
            "2026-11-15T20:00:00.000Z".into(),
            Some("book".into()),
            Some(42),
            vec![PimdirReceiptRefusal {
                collection: "book".into(),
                source: "carddav".into(),
                reason: "403 Forbidden".into(),
                refused_at: "2026-11-15T20:05:00.000Z".into(),
            }],
            None,
        );

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["state"], "applied");
        assert_eq!(
            json["refusals"],
            serde_json::json!([{
                "collection": "book",
                "source": "carddav",
                "reason": "403 Forbidden",
                "refusedAt": "2026-11-15T20:05:00.000Z",
            }])
        );
        assert_eq!(
            row.to_string(),
            format!(
                "Row {STORE}.1.12 was applied in book, on card 42\n\
                 Refused by carddav in book at 2026-11-15T20:05:00.000Z: 403 Forbidden\n"
            )
        );
    }

    /// A row applied on an address book deleted since keeps its receipt and
    /// names no collection (pimdir STORAGE §15.4).
    #[test]
    fn an_applied_row_of_a_deleted_addressbook_names_no_collection() {
        let row = PimdirQueueShowOutput::applied(
            &handle(4),
            "2026-11-13T09:00:00Z".into(),
            None,
            None,
            Vec::new(),
            None,
        );

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["state"], "applied");
        assert_eq!(json["collection"], serde_json::Value::Null);
        assert!(row.to_string().contains("deleted since"), "{row}");
    }
}
