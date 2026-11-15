//! # ContactCard create command
//!
//! Creates a ContactCard from a raw JSContact body.

use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::contact_card::{JmapContactCard, set::JmapContactCardSetArgs};
use pimalaya_cli::printer::Printer;

use crate::jmap::{
    client::JmapClient, error::set_error, input::JsonArg, render::JmapContactCardOutput,
};

/// Create a ContactCard from a raw JSContact body (`ContactCard/set`).
///
/// JSON output: the raw JMAP ContactCard the server created.
#[derive(Debug, Parser)]
pub struct JmapContactCardAddCommand {
    /// AddressBook the card is created in.
    #[arg(short = 'k', long, value_name = "ADDRESS-BOOK")]
    pub in_address_book: String,
    #[command(flatten)]
    pub json: JsonArg,
}

impl JmapContactCardAddCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let card = self.json.read()?;

        let contact_card = JmapContactCard {
            id: None,
            address_book_ids: BTreeMap::from([(self.in_address_book, true)]),
            card,
        };
        let create = BTreeMap::from([("c0".to_string(), contact_card)]);
        let args = JmapContactCardSetArgs {
            create: Some(create),
            ..Default::default()
        };
        let out = client.contact_card_set(args)?;

        if let Some(err) = out.not_created.into_values().next() {
            return Err(set_error("ContactCard create rejected", &err));
        }
        let created = out
            .created
            .into_values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("ContactCard create returned no object"))?;

        printer.out(JmapContactCardOutput(created))
    }
}
