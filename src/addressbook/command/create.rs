use anyhow::Result;
use clap::Parser;
use io_addressbook::Addressbook;
use pimalaya_toolbox::terminal::printer::{Message, Printer};

use crate::{account::Account, client::Client};

/// Create all folders.
///
/// This command allows you to create all exsting folders.
#[derive(Debug, Parser)]
pub struct CreateAddressbookCommand {
    pub name: Option<String>,
    #[arg(long, short, alias = "desc")]
    pub description: Option<String>,
    #[arg(long, short = 'C')]
    pub color: Option<String>,
}

impl CreateAddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let mut addressbook = Addressbook::new();
        addressbook.display_name = self.name;
        addressbook.description = self.description;
        addressbook.color = self.color;

        client.create_addressbook(addressbook)?;
        printer.out(Message::new("Addressbook successfully created"))
    }
}
