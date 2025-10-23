use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::{account::Account, addressbook::table::AddressbooksTable, client::Client};

/// List all addressbooks.
///
/// This command allows you to list all exsting addressbooks.
#[derive(Debug, Parser)]
pub struct ListAddressbooksCommand;

impl ListAddressbooksCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let addressbooks = client.list_addressbooks()?;
        let table = AddressbooksTable::from(addressbooks);
        printer.out(table)
    }
}
