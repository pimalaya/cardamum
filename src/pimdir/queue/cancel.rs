//! # pimdir queue cancel
//!
//! The `pimdir queue cancel` command, retracting one queued or parked
//! action before its owner applies it.

use std::{
    fmt,
    io::{IsTerminal, stdin},
};

use anyhow::{Result, bail};
use clap::Parser;
use io_pimdir::{client::producer::PimdirActionStatus, codec::PimdirActionHandle};
use pimalaya_cli::{error::CodedError, printer::Printer, prompt, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    error::ErrorCode,
    pimdir::backend::{PimdirBackend, uncancellable},
};

/// Cancel one queued or parked action, by the row id `queue list` prints,
/// or by a row handle naming its store (`<store-id>.<incarnation>.<id>`),
/// refused under `store-reset` when the store is another one.
///
/// This is the only way back for a queued creation: a staged update is
/// undone by updating again, where a card that does not exist yet cannot
/// be deleted. Only a row of the account's address books is taken, a row
/// it cannot cancel refused before anything is asked, and with no terminal
/// to answer `--yes` is required.
///
/// An unconfirmed row, one the sync engine sent without learning its
/// outcome, is cancelled the same way, which undoes nothing: it may have
/// taken effect, and both the question and the answer say so.
///
/// Cancelling is the store owner's write, so this takes that role for the
/// length of the call and fails while a sync holds it. A store another
/// version of the sync engine wrote is refused rather than reshaped.
///
/// JSON output: `{"row", "handle", "kind", "unconfirmed"}`.
#[derive(Debug, Parser)]
pub struct PimdirQueueCancelCommand {
    /// Row id of the queued action, as `pimdir queue list` prints it, or
    /// its handle.
    #[arg(value_name = "ROW")]
    pub id: PimdirActionHandle,
    /// Do not ask for confirmation.
    #[arg(long, short)]
    pub yes: bool,
}

impl PimdirQueueCancelCommand {
    /// Retracts the queued action the row id names.
    pub fn execute(self, printer: &mut impl Printer, backend: &PimdirBackend) -> Result<()> {
        let (named, unconfirmed) = match backend.queue_row(&self.id)? {
            PimdirActionStatus::Pending { kind, .. } | PimdirActionStatus::Parked { kind, .. } => {
                (kind, false)
            }
            PimdirActionStatus::Unconfirmed { kind, .. } => (kind, true),
            status => return Err(uncancellable(&self.id, &status)),
        };
        let question = match unconfirmed {
            true => format!(
                "The {} queued as row {} was sent and may have taken effect, which \
                 cancelling does not undo. Cancel it?",
                sanitize::text(&named),
                self.id
            ),
            false => format!(
                "Cancel the {} queued as row {}?",
                sanitize::text(&named),
                self.id
            ),
        };
        if !self.yes && !stdin().is_terminal() {
            let message = format!(
                "Cannot ask to cancel row {} with no terminal: pass --yes",
                self.id
            );
            return Err(CodedError::new(ErrorCode::ConfirmationRequired, message).into());
        }

        if !self.yes && !prompt::bool(question, false)? {
            bail!("Cancellation aborted");
        }

        let handle = backend.named(&self.id)?;
        let kind = backend.cancel_queued(&self.id)?;
        printer.out(PimdirQueueCancelOutput {
            row: handle.id,
            handle: handle.to_string(),
            kind,
            unconfirmed,
        })
    }
}

/// The `pimdir queue cancel` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirQueueCancelOutput {
    /// The queue row id cancelled.
    pub row: i64,
    /// The row's handle, its id under the store's identity (pimdir STORAGE
    /// §4.5): the handle given, or the bare id given named under this
    /// store's identity.
    pub handle: String,
    /// The action kind it held: `add`, `update`, `remove`, ...
    pub kind: String,
    /// Whether the row was unconfirmed, its request sent and its outcome
    /// unknown: the action may have taken effect all the same.
    pub unconfirmed: bool,
}

impl fmt::Display for PimdirQueueCancelOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = sanitize::text(&self.kind);
        let id = &self.handle;
        match self.unconfirmed {
            true => writeln!(
                f,
                "Unconfirmed {kind} cancelled (row {id}); it was sent and may have taken effect"
            ),
            false => writeln!(f, "Queued {kind} cancelled (row {id})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cancelling an unconfirmed row drops it from the queue and undoes
    /// nothing its performer sent (pimdir STORAGE §15.2), so the output
    /// never reads as a plain cancellation.
    #[test]
    fn cancelling_an_unconfirmed_row_says_it_may_have_taken_effect() {
        let cancelled = PimdirQueueCancelOutput {
            row: 9,
            handle: "e2646df031cd876334410c4df14ebd86.1.9".into(),
            kind: "add".into(),
            unconfirmed: true,
        };

        let json = serde_json::to_value(&cancelled).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "row": 9,
                "handle": "e2646df031cd876334410c4df14ebd86.1.9",
                "kind": "add",
                "unconfirmed": true,
            })
        );
        let line = cancelled.to_string();
        assert!(line.contains("may have taken effect"), "{line}");
    }
}
