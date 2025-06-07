use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::{clap::args::AccountArg, printer::Printer};

use crate::{account::Account, addressbook::table::AddressbooksTable, client::Client};

/// List all folders.
///
/// This command allows you to list all exsting folders.
#[derive(Debug, Parser)]
pub struct ListAddressbooksCommand {
    #[command(flatten)]
    pub account: AccountArg,
}

impl ListAddressbooksCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(account)?;

        let addressbooks = client.list_addressbooks()?;
        let table = AddressbooksTable::from(addressbooks);
        printer.out(table)
    }
}
