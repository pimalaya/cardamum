use anyhow::Result;
use clap::Parser;
use io_people::v1::rest::contact_groups::PeopleContactGroup;
use pimalaya_cli::printer::Printer;

use crate::google::{client::GoogleClient, render::GroupReport};

/// Rename a contact group. The People API guards the update on the
/// group's current etag, fetched first.
///
/// JSON output: the raw People group after the update.
#[derive(Debug, Parser)]
pub struct GoogleContactGroupUpdateCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// New name.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl GoogleContactGroupUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let resource_name = format!("contactGroups/{}", self.group_id);
        let current = client
            .contact_group_get(&resource_name, None, &[])?
            .response;

        let group = PeopleContactGroup {
            resource_name,
            etag: current.etag,
            name: Some(self.name),
            ..Default::default()
        };
        let updated = client.contact_group_update(&group, &[], &[])?.response;

        printer.out(GroupReport(updated))
    }
}
