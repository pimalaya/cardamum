use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{arg::AddressbookIdArg, card::vcard::VcardArg, client::AddressbookClient};

/// Append a vCard to the given addressbook.
#[derive(Debug, Parser)]
pub struct CardCreateCommand {
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    #[command(flatten)]
    pub vcard: VcardArg,
}

impl CardCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let contents = self.vcard.read()?;
        let id = client.create_card(&addressbook_id, contents)?;

        printer.out(Message::new(format!("Card `{id}` successfully created")))
    }
}
