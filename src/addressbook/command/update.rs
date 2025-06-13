use anyhow::Result;
use clap::Parser;
use io_addressbook::Addressbook;
use pimalaya_toolbox::terminal::printer::{Message, Printer};

use crate::{account::Account, client::Client};

/// Update all folders.
///
/// This command allows you to update all exsting folders.
#[derive(Debug, Parser)]
pub struct UpdateAddressbookCommand {
    pub id: String,
    #[arg(long, short)]
    pub name: Option<String>,
    #[arg(long, short, alias = "desc")]
    pub description: Option<String>,
    #[arg(long, short = 'C')]
    pub color: Option<String>,
}

impl UpdateAddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let addressbook = Addressbook {
            id: self.id,
            display_name: self.name,
            description: self.description,
            color: self.color,
        };

        client.update_addressbook(addressbook)?;
        printer.out(Message::new("Addressbook successfully updated"))
    }
}
