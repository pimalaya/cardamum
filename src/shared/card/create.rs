use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::{card::vcard::VcardArg, client::AddressbookClient};

/// Append a vCard to the given addressbook.
#[derive(Debug, Parser)]
pub struct CardCreateCommand {
    /// Identifier of the destination addressbook. Falls back to the
    /// `addressbook.default` config when omitted.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub addressbook_id: Option<String>,
    #[command(flatten)]
    pub vcard: VcardArg,
}

impl CardCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook_id)?;
        let contents = self.vcard.read()?;
        let id = client.create_card(&addressbook_id, contents)?;

        printer.out(Message::new(format!("Card `{id}` successfully created")))
    }
}
