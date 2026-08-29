//! # ContactCard update command
//!
//! Updates a ContactCard from a raw JSContact patch.

use std::collections::BTreeMap;

use anyhow::{Result, bail};
use clap::Parser;
use io_jmap::rfc9610::contact_card::set::{JmapContactCardPatch, JmapContactCardSetArgs};
use pimalaya_cli::printer::{Message, Printer};

use crate::jmap::{
    client::JmapClient, error::format_set_error, input::JsonArg, render::CardReport,
};

/// Update a ContactCard from a raw JSContact patch (`ContactCard/set`).
///
/// The JSON's top-level keys are the patch; JMAP spells JSON pointers
/// as dotted keys.
///
/// JSON output: the raw JMAP ContactCard after the update, or
/// `{"message": "..."}` when the server returns nothing to show (most
/// do: a `set` response echoes only the properties the server itself
/// decided).
#[derive(Debug, Parser)]
pub struct JmapContactCardUpdateCommand {
    /// ContactCard id.
    #[arg(value_name = "CARD-ID")]
    pub id: String,
    #[command(flatten)]
    pub json: JsonArg,
}

impl JmapContactCardUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let patch = self.json.read()?.into_iter().collect();

        let update = BTreeMap::from([(self.id.clone(), JmapContactCardPatch(patch))]);
        let args = JmapContactCardSetArgs {
            update: Some(update),
            ..Default::default()
        };
        let out = client.contact_card_set(args)?;

        if let Some(err) = out.not_updated.into_values().next() {
            bail!("ContactCard update rejected{}", format_set_error(&err));
        }

        // NOTE: the server may answer with no object at all, or with one
        // carrying only its own bookkeeping (Fastmail returns `updated`
        // and a blob id). Both mean "nothing to show beyond the
        // success", so say that rather than print an empty block.
        let updated = out
            .updated
            .into_values()
            .next()
            .flatten()
            .map(CardReport)
            .filter(|report| !report.is_empty());

        match updated {
            Some(report) => printer.out(report),
            None => printer.out(Message::new(format!(
                "ContactCard `{}` successfully updated",
                self.id
            ))),
        }
    }
}
