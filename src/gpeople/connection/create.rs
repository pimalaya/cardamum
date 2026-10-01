//! # Connection create command
//!
//! Creates a contact from a raw People person JSON body
//! (`people.createContact`).

use anyhow::{Context, Result};
use clap::Parser;
use io_gpeople::v1::rest::people::{GpeoplePerson, vcard::GPEOPLE_PERSON_VCARD_FIELDS};
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, input::PersonJsonArg, render::GpeoplePersonOutput};

/// Create a contact from a raw People person JSON body.
///
/// The contact lands in `myContacts`: add group memberships with
/// `contact-group members --add`.
///
/// JSON output: the raw People person the server created.
#[derive(Debug, Parser)]
pub struct GpeopleConnectionCreateCommand {
    /// The person JSON body: a file path, inline JSON, or `-`.
    #[command(flatten)]
    pub json: PersonJsonArg,
}

impl GpeopleConnectionCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let value = self.json.read()?;
        let person: GpeoplePerson =
            serde_json::from_value(value).context("Invalid People person JSON")?;

        let created = client
            .contact_create(&person, GPEOPLE_PERSON_VCARD_FIELDS, &[])?
            .response;

        printer.out(GpeoplePersonOutput(created))
    }
}
