//! # Contact group update command
//!
//! Renames a contact group (`contactGroups.update`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::rest::contact_groups::GpeopleContactGroup;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, render::GpeopleContactGroupOutput};

/// Rename a contact group.
///
/// The People API guards the update on the group's current etag,
/// fetched first.
///
/// JSON output: the raw People group after the update.
#[derive(Debug, Parser)]
pub struct GpeopleContactGroupUpdateCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// New name.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl GpeopleContactGroupUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let resource_name = format!("contactGroups/{}", self.group_id);
        let current = client
            .contact_group_get(&resource_name, None, &[])?
            .response;

        let group = GpeopleContactGroup {
            resource_name,
            etag: current.etag,
            name: Some(self.name),
            ..Default::default()
        };
        let updated = client.contact_group_update(&group, &[], &[])?.response;

        printer.out(GpeopleContactGroupOutput(updated))
    }
}
