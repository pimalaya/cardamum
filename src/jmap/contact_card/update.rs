use std::collections::BTreeMap;

use anyhow::{Result, bail};
use clap::Parser;
use io_jmap::rfc9610::contact_card::set::{JmapContactCardPatch, JmapContactCardSetArgs};
use pimalaya_cli::printer::{Message, Printer};

use crate::jmap::{client::JmapClient, input::JsonArg, render::CardReport};

/// Update a ContactCard from a raw JSContact patch (`ContactCard/set`
/// update). The JSON's top-level keys are the patch (JSON pointers are
/// supported by JMAP as dotted keys).
///
/// JSON output: the raw JMAP ContactCard after the update (or a message
/// when the server returns no object).
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
            bail!("ContactCard update rejected: {err:?}");
        }

        match out.updated.into_values().next().flatten() {
            Some(card) => printer.out(CardReport(card)),
            None => printer.out(Message::new(format!(
                "ContactCard `{}` successfully updated",
                self.id
            ))),
        }
    }
}
