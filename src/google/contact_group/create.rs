use anyhow::Result;
use clap::Parser;
use io_people::v1::rest::contact_groups::PeopleContactGroup;
use pimalaya_cli::printer::Printer;

use crate::google::{client::GoogleClient, render::GroupReport};

/// Create a contact group.
///
/// JSON output: the raw People group the server created.
#[derive(Debug, Parser)]
pub struct GoogleContactGroupCreateCommand {
    /// Name of the group to create.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl GoogleContactGroupCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let group = PeopleContactGroup {
            name: Some(self.name),
            ..Default::default()
        };
        let created = client.contact_group_create(&group, &[])?.response;

        printer.out(GroupReport(created))
    }
}
