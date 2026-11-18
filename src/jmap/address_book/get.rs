//! # AddressBook get command
//!
//! Fetches AddressBooks by id, or every one of them.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, raw, render::JmapAddressBooksOutput};

/// Get JMAP AddressBooks by id (`AddressBook/get`); omit ids for all.
///
/// JSON output: `{"list": [<AddressBook as the server sent it>...],
/// "state"}`.
#[derive(Debug, Parser)]
pub struct JmapAddressBookGetCommand {
    /// AddressBook id(s) to fetch. Omit to fetch all.
    #[arg(value_name = "ID")]
    pub ids: Vec<String>,
}

impl JmapAddressBookGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let ids = (!self.ids.is_empty()).then_some(self.ids);
        let out = raw::get(&mut client, "AddressBook", ids)?;

        printer.out(JmapAddressBooksOutput {
            books: out.list,
            state: out.state,
        })
    }
}
