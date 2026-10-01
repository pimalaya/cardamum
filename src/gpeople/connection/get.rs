//! # Connection get command
//!
//! Reads one contact by id (`people.get`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::rest::people::vcard::GPEOPLE_PERSON_VCARD_FIELDS;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, render::GpeoplePersonOutput};

/// GET a contact by id.
///
/// JSON output: the raw People person object.
#[derive(Debug, Parser)]
pub struct GpeopleConnectionGetCommand {
    /// Person id (the segment after `people/`).
    #[arg(value_name = "PERSON-ID")]
    pub person_id: String,
}

impl GpeopleConnectionGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let resource_name = format!("people/{}", self.person_id);
        let person = client
            .person_get(&resource_name, GPEOPLE_PERSON_VCARD_FIELDS, &[])?
            .response;

        printer.out(GpeoplePersonOutput(person))
    }
}
