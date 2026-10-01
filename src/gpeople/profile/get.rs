//! # Profile get command
//!
//! Reads the signed-in user's own person (`people/me`).

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, project, render::GpeoplePersonOutput};

/// GET the signed-in user (`people/me`).
///
/// JSON output: the raw People person object.
#[derive(Debug, Parser)]
pub struct GpeopleProfileGetCommand;

impl GpeopleProfileGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let person = client
            .person_get("people/me", project::READ_FIELDS, &[])?
            .response;
        printer.out(GpeoplePersonOutput(person))
    }
}
