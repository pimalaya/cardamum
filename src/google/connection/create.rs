use anyhow::{Context, Result};
use clap::Parser;
use io_people::v1::rest::people::PeoplePerson;
use pimalaya_cli::printer::Printer;

use crate::google::{client::GoogleClient, input::PersonJsonArg, project, render::PersonReport};

/// Create a contact from a raw People person JSON body
/// (`people.createContact`). It lands in `myContacts`; add group
/// memberships with `contact-group members --add`.
///
/// JSON output: the raw People person the server created.
#[derive(Debug, Parser)]
pub struct GoogleConnectionCreateCommand {
    #[command(flatten)]
    pub json: PersonJsonArg,
}

impl GoogleConnectionCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let value = self.json.read()?;
        let person: PeoplePerson =
            serde_json::from_value(value).context("Invalid People person JSON")?;

        let created = client
            .contact_create(&person, project::READ_FIELDS, &[])?
            .response;

        printer.out(PersonReport(created))
    }
}
