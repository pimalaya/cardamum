//! # Profile get command
//!
//! Reads the signed-in user's own person (`people/me`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::{query::to_field_mask, rest::people::vcard::GPEOPLE_PERSON_VCARD_FIELDS};
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, render::GpeoplePersonGetOutput};

/// GET the signed-in user (`people/me`).
///
/// JSON output: the person as People sent it, every property kept.
#[derive(Debug, Parser)]
pub struct GpeopleProfileGetCommand;

impl GpeopleProfileGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let fields = [("personFields", to_field_mask(GPEOPLE_PERSON_VCARD_FIELDS))];
        let person = client.raw_get("people", "me", &fields)?;
        printer.out(GpeoplePersonGetOutput(person))
    }
}
