//! # Connection update command
//!
//! Updates a contact from a raw People person JSON body
//! (`people.updateContact`).

use anyhow::{Context, Result};
use clap::Parser;
use io_gpeople::v1::rest::people::{GpeoplePerson, vcard::GPEOPLE_PERSON_VCARD_FIELDS};
use pimalaya_cli::printer::Printer;

use crate::gpeople::{
    client::{GpeopleClient, person_refusal},
    input::{PersonJsonArg, update_fields_from_json},
    render::GpeoplePersonOutput,
};

/// Update a contact from a raw People person JSON body.
///
/// The update mask is derived from the JSON's top-level keys. The etag
/// the JSON names guards the write; without one, the current etag is
/// fetched to guard it.
///
/// JSON output: the raw People person after the update.
#[derive(Debug, Parser)]
pub struct GpeopleConnectionUpdateCommand {
    /// Person id (the segment after `people/`).
    #[arg(value_name = "PERSON-ID")]
    pub person_id: String,
    /// The person JSON body: a file path, inline JSON, or `-`.
    #[command(flatten)]
    pub json: PersonJsonArg,
}

impl GpeopleConnectionUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let value = self.json.read()?;
        let fields = update_fields_from_json(&value);
        let mut person: GpeoplePerson =
            serde_json::from_value(value).context("Invalid People person JSON")?;

        let resource_name = format!("people/{}", self.person_id);

        if person.etag.is_empty() {
            person.etag = client
                .person_get(&resource_name, GPEOPLE_PERSON_VCARD_FIELDS, &[])
                .map_err(person_refusal)?
                .response
                .etag;
        }

        person.resource_name = resource_name;

        let updated = client
            .contact_update(&person, &fields, GPEOPLE_PERSON_VCARD_FIELDS, &[])
            .map_err(person_refusal)?
            .response;

        printer.out(GpeoplePersonOutput(updated))
    }
}
