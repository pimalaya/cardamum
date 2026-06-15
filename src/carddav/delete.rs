use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::carddav::client::CarddavClient;

/// Delete an addressbook collection on the server.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CarddavAddressbookDeleteCommand {
    /// Identifier (last URL segment) of the addressbook to delete.
    #[arg(value_name = "ID")]
    pub id: String,
}

impl CarddavAddressbookDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        client.delete_addressbook(&self.id)?;

        printer.out(Message::new(format!(
            "Addressbook `{}` successfully deleted",
            self.id
        )))
    }
}
