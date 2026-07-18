use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::address_book::get::JmapAddressBookGetOptions;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, render::BooksReport};

/// Get JMAP AddressBooks by id (`AddressBook/get`); omit ids for all.
///
/// JSON output: `{"list": [<raw JMAP AddressBook>...], "state"}`.
#[derive(Debug, Parser)]
pub struct JmapAddressBookGetCommand {
    /// AddressBook id to fetch; repeatable. Omit to fetch all.
    #[arg(long = "id", value_name = "ID")]
    pub ids: Vec<String>,
}

impl JmapAddressBookGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.addressbooks_list_table_id_color();

        let opts = JmapAddressBookGetOptions {
            ids: (!self.ids.is_empty()).then_some(self.ids),
            ..Default::default()
        };
        let out = client.address_book_get(opts)?;

        printer.out(BooksReport {
            preset,
            id_color,
            books: out.address_books,
            state: out.new_state,
        })
    }
}
