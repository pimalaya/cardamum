use std::process::exit;

use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::{
    printer::{Message, Printer},
    prompt,
};

use crate::{account::Account, client::Client};

/// Delete all folders.
///
/// This command allows you to delete all exsting folders.
#[derive(Debug, Parser)]
pub struct DeleteAddressbookCommand {
    pub id: String,
    #[arg(long, short)]
    pub yes: bool,
}

impl DeleteAddressbookCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        if !self.yes {
            let confirm = "Do you really want to delete this addressbook";
            let confirm = format!("{confirm}? All contacts will be definitely deleted.");

            if !prompt::bool(confirm, false)? {
                exit(0);
            };
        };

        let mut client = Client::new(&account)?;

        client.delete_addressbook(self.id)?;
        printer.out(Message::new("Addressbook successfully deleted"))
    }
}
