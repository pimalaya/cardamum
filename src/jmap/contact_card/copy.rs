use std::collections::BTreeMap;

use anyhow::{Result, bail};
use clap::Parser;
use io_jmap::rfc9610::contact_card::copy::JmapContactCardCopyArgs;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, render::CardReport};

/// Copy a ContactCard from another JMAP account into an AddressBook of
/// this one (`ContactCard/copy`).
///
/// JSON output: the raw JMAP ContactCard created in this account.
#[derive(Debug, Parser)]
pub struct JmapContactCardCopyCommand {
    /// Source account id to copy from.
    #[arg(short = 'a', long, value_name = "ACCOUNT-ID")]
    pub from_account: String,
    /// Destination AddressBook id in this account.
    #[arg(short = 'k', long, value_name = "ADDRESS-BOOK")]
    pub to_address_book: String,
    /// ContactCard id in the source account.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
}

impl JmapContactCardCopyCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let cards = BTreeMap::from([(
            "c0".to_string(),
            JmapContactCardCopyArgs {
                id: self.card_id,
                address_book_ids: BTreeMap::from([(self.to_address_book, true)]),
            },
        )]);
        let out = client.contact_card_copy(self.from_account, cards)?;

        if let Some(err) = out.not_created.into_values().next() {
            bail!("ContactCard copy rejected: {err:?}");
        }
        let created = out
            .created
            .into_values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("ContactCard copy returned no object"))?;

        printer.out(CardReport(created))
    }
}
