//! # Connection delete command
//!
//! Deletes a contact by id (`people.deleteContact`).

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{gpeople::client::GpeopleClient, shared::receipt::IdReceipt};

/// Delete a contact by id (People `people.deleteContact`).
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct GpeopleConnectionDeleteCommand {
    /// Person id (the segment after `people/`).
    #[arg(value_name = "PERSON-ID")]
    pub person_id: String,
}

impl GpeopleConnectionDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let resource_name = format!("people/{}", self.person_id);
        client.contact_delete(&resource_name)?;

        printer.out(IdReceipt {
            id: self.person_id,
            done: "Contact deleted",
        })
    }
}
