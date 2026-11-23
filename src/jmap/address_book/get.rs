//! # AddressBook get command
//!
//! Fetches AddressBooks by id, or every one of them.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    error::{ErrorCode, partial_when},
    jmap::{client::JmapClient, raw, render::JmapAddressBooksOutput},
};

/// Get JMAP AddressBooks by id (`AddressBook/get`); omit ids for all.
///
/// JSON output: `{"list": [<AddressBook as the server sent it>...],
/// "state", "errors"}`, `errors` holding an `addressbook-not-found` error
/// for each id the server found nothing for, `[]` when none, the command
/// then exiting 3.
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
        let errors = out.not_found_errors(ErrorCode::AddressbookNotFound, "AddressBook");

        printer.out(JmapAddressBooksOutput {
            books: out.list,
            state: out.state,
            errors: errors.clone(),
        })?;

        partial_when(&errors)
    }
}
