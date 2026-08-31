//! # Contact group create command
//!
//! Creates a contact group (`contactGroups.create`).

use anyhow::Result;
use clap::Parser;
use io_people::v1::rest::contact_groups::PeopleContactGroup;
use pimalaya_cli::printer::Printer;

use crate::people::{client::PeopleClient, render::PeopleContactGroupOutput};

/// Create a contact group.
///
/// JSON output: the raw People group the server created.
#[derive(Debug, Parser)]
pub struct PeopleContactGroupCreateCommand {
    /// Name of the group to create.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl PeopleContactGroupCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: PeopleClient) -> Result<()> {
        let group = PeopleContactGroup {
            name: Some(self.name),
            ..Default::default()
        };
        let created = client.contact_group_create(&group, &[])?.response;

        printer.out(PeopleContactGroupOutput(created))
    }
}
