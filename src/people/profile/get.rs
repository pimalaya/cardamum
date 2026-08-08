use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::people::{client::PeopleClient, project, render::PersonReport};

/// GET the signed-in user (People `people/me`).
///
/// JSON output: the raw People person object.
#[derive(Debug, Parser)]
pub struct PeopleProfileGetCommand;

impl PeopleProfileGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: PeopleClient) -> Result<()> {
        let person = client
            .person_get("people/me", project::READ_FIELDS, &[])?
            .response;
        printer.out(PersonReport(person))
    }
}
