//! # Contact delete command
//!
//! Deletes a contact by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    msgraph::client::{MsgraphClient, contact_refusal},
    shared::receipt::IdReceipt,
};

/// Delete a contact by id.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct MsgraphContactDeleteCommand {
    /// Contact id.
    #[arg(value_name = "CONTACT-ID")]
    pub contact_id: String,
}

impl MsgraphContactDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        client
            .contact_delete(&self.contact_id)
            .map_err(contact_refusal)?;

        printer.out(IdReceipt {
            id: self.contact_id,
            done: "Contact deleted",
        })
    }
}
