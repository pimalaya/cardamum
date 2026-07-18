use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::google::{client::GoogleClient, project, render::PersonReport};

/// GET a contact by id (People `people.get`).
///
/// JSON output: the raw People person object.
#[derive(Debug, Parser)]
pub struct GoogleConnectionGetCommand {
    /// Person id (the segment after `people/`).
    #[arg(value_name = "PERSON-ID")]
    pub person_id: String,
}

impl GoogleConnectionGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let resource_name = format!("people/{}", self.person_id);
        let person = client
            .person_get(&resource_name, project::READ_FIELDS, &[])?
            .response;

        printer.out(PersonReport(person))
    }
}
