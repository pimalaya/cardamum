use std::collections::BTreeMap;

use anyhow::{Result, bail};
use clap::Parser;
use io_jmap::rfc9610::contact_card::{JmapContactCard, set::JmapContactCardSetArgs};
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, input::JsonArg, render::CardReport};

/// Create a ContactCard from a raw JSContact JSON body
/// (`ContactCard/set` create).
///
/// JSON output: the raw JMAP ContactCard the server created.
#[derive(Debug, Parser)]
pub struct JmapContactCardCreateCommand {
    /// AddressBook the card is created in.
    #[arg(short = 'k', long, value_name = "ADDRESS-BOOK")]
    pub in_address_book: String,
    #[command(flatten)]
    pub json: JsonArg,
}

impl JmapContactCardCreateCommand {
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
            bail!("ContactCard create rejected: {err:?}");
        }
        let created = out
            .created
            .into_values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("ContactCard create returned no object"))?;

        printer.out(CardReport(created))
    }
}
