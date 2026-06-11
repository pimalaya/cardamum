use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::shared::client::AddressbookClient;

/// Delete the given addressbook and every card it contains.
#[derive(Debug, Parser)]
pub struct AddressbookDeleteCommand {
    /// Backend-assigned identifier of the addressbook to delete.
    #[arg(value_name = "ADDRESSBOOK-ID")]
    pub id: String,
}

impl AddressbookDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        client.delete_addressbook(&self.id)?;

        let msg = format!("Addressbook `{}` successfully deleted", self.id);
        printer.out(Message::new(msg))
    }
}
