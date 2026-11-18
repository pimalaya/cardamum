//! # Connection get command
//!
//! Reads one contact by id (`people.get`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::{query::to_field_mask, rest::people::vcard::GPEOPLE_PERSON_VCARD_FIELDS};
use pimalaya_cli::printer::Printer;

use crate::gpeople::{
    client::{GpeopleClient, person_refusal},
    render::GpeoplePersonGetOutput,
};

/// GET a contact by id.
///
/// JSON output: the person as People sent it, every property kept.
#[derive(Debug, Parser)]
pub struct GpeopleConnectionGetCommand {
    /// Person id (the segment after `people/`).
    #[arg(value_name = "PERSON-ID")]
    pub person_id: String,
}

impl GpeopleConnectionGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let fields = [("personFields", to_field_mask(GPEOPLE_PERSON_VCARD_FIELDS))];
        let person = client
            .raw_get("people", &self.person_id, &fields)
            .map_err(person_refusal)?;

        printer.out(GpeoplePersonGetOutput(person))
    }
}
