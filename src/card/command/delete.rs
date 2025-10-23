use std::process;

use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::{
    printer::{Message, Printer},
    prompt,
};

use crate::{account::Account, client::Client};

/// Delete a card.
///
/// This command allows you to delete a vCard from an addressbook.
#[derive(Debug, Parser)]
pub struct DeleteCardCommand {
    /// The identifier of the addressbook where the vCard should be
    /// deleted from.
    #[arg(name = "ADDRESSBOOK-ID")]
    pub addressbook_id: String,

    /// The identifier of the vCard to delete.
    #[arg(name = "CARD-ID")]
    pub id: String,

    #[arg(long, short)]
    pub yes: bool,
}

impl DeleteCardCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        if !self.yes {
            let confirm = "Do you really want to delete this card?";

            if !prompt::bool(confirm, false)? {
                process::exit(0);
            };
        };

        let mut client = Client::new(&account)?;
        client.delete_card(self.addressbook_id, self.id)?;
        printer.out(Message::new("Card successfully deleted"))
    }
}
