//! # Addressbook delete command
//!
//! Deletes an addressbook and every card it contains.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::shared::{
    client::AddressbookClient,
    receipt::{Write, WriteReceipt},
};

/// Delete the given addressbook and every card it contains.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct AddressbookDeleteCommand {
    /// Addressbook to delete.
    ///
    /// Mandatory: unlike the other shared commands, a deletion never falls
    /// back to the `addressbook.default` config.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub id: String,
}

impl AddressbookDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        client.delete_addressbook(&self.id)?;

        printer.out(WriteReceipt {
            id: self.id,
            queued: client.queues_writes(),
            what: "Addressbook",
            write: Write::Delete,
        })
    }
}
