use anyhow::{Context, Result};
use clap::Parser;
use io_people::v1::rest::people::PeoplePerson;
use pimalaya_cli::printer::Printer;

use crate::google::{
    client::GoogleClient,
    input::{PersonJsonArg, update_fields_from_json},
    project,
    render::PersonReport,
};

/// Update a contact from a raw People person JSON body
/// (`people.updateContact`). The update mask is derived from the JSON's
/// top-level keys, and the current etag is fetched to guard the write.
///
/// JSON output: the raw People person after the update.
#[derive(Debug, Parser)]
pub struct GoogleConnectionUpdateCommand {
    /// Person id (the segment after `people/`).
    #[arg(value_name = "PERSON-ID")]
    pub person_id: String,
    #[command(flatten)]
    pub json: PersonJsonArg,
}

impl GoogleConnectionUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let value = self.json.read()?;
        let fields = update_fields_from_json(&value);
        let mut person: PeoplePerson =
            serde_json::from_value(value).context("Invalid People person JSON")?;

        let resource_name = format!("people/{}", self.person_id);
        let current = client
            .person_get(&resource_name, project::READ_FIELDS, &[])?
            .response;
        person.resource_name = resource_name;
        person.etag = current.etag;

        let updated = client
            .contact_update(&person, &fields, project::READ_FIELDS, &[])?
            .response;

        printer.out(PersonReport(updated))
    }
}
