use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::google::client::GoogleClient;

/// Delete a contact by id (People `people.deleteContact`).
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct GoogleConnectionDeleteCommand {
    /// Person id (the segment after `people/`).
    #[arg(value_name = "PERSON-ID")]
    pub person_id: String,
}

impl GoogleConnectionDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let resource_name = format!("people/{}", self.person_id);
        client.contact_delete(&resource_name)?;

        printer.out(Message::new(format!(
            "Contact `{}` successfully deleted",
            self.person_id
        )))
    }
}
