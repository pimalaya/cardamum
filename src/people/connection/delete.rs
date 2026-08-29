//! # Connection delete command
//!
//! Deletes a contact by id (`people.deleteContact`).

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::people::client::PeopleClient;

/// Delete a contact by id (People `people.deleteContact`).
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct PeopleConnectionDeleteCommand {
    /// Person id (the segment after `people/`).
    #[arg(value_name = "PERSON-ID")]
    pub person_id: String,
}

impl PeopleConnectionDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: PeopleClient) -> Result<()> {
        let resource_name = format!("people/{}", self.person_id);
        client.contact_delete(&resource_name)?;

        printer.out(Message::new(format!(
            "Contact `{}` successfully deleted",
            self.person_id
        )))
    }
}
