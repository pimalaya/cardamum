//! # ContactCard copy command
//!
//! Copies ContactCards from another account of the same JMAP server.

use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::contact_card::copy::JmapContactCardCopyArgs;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, set::JmapSetReceipt};

/// Copy ContactCards from another JMAP account (`ContactCard/copy`).
///
/// The cards land in the AddressBooks of this account that
/// `--to-address-book` names.
///
/// JSON output: `{"ids", "errors"}`, the ids of the copies made, then
/// one item error per card the server refused, keyed by the id given;
/// a refused card exits 3.
#[derive(Debug, Parser)]
pub struct JmapContactCardCopyCommand {
    /// ContactCard id(s) in the source account.
    #[arg(value_name = "CARD-ID", required = true)]
    pub ids: Vec<String>,
    /// Source account id to copy from.
    #[arg(long, value_name = "ACCOUNT-ID")]
    pub from_account: String,
    /// Destination AddressBook id(s) in this account; repeatable.
    #[arg(long, value_name = "ADDRESS-BOOK", required = true)]
    pub to_address_book: Vec<String>,
}

impl JmapContactCardCopyCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let address_book_ids: BTreeMap<String, bool> = self
            .to_address_book
            .into_iter()
            .map(|id| (id, true))
            .collect();

        let cards: BTreeMap<String, JmapContactCardCopyArgs> = self
            .ids
            .into_iter()
            .map(|id| {
                let args = JmapContactCardCopyArgs {
                    id: id.clone(),
                    address_book_ids: address_book_ids.clone(),
                };
                (id, args)
            })
            .collect();

        let out = client.contact_card_copy(self.from_account, cards)?;

        let ids: Vec<String> = out
            .created
            .into_values()
            .filter_map(|card| card.id)
            .collect();
        JmapSetReceipt::new("ContactCards copied", ids, out.not_created).print(printer)
    }
}
