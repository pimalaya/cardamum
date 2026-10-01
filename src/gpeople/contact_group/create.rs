//! # Contact group create command
//!
//! Creates a contact group (`contactGroups.create`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::rest::contact_groups::GpeopleContactGroup;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, render::GpeopleContactGroupOutput};

/// Create a contact group.
///
/// JSON output: the raw People group the server created.
#[derive(Debug, Parser)]
pub struct GpeopleContactGroupCreateCommand {
    /// Name of the group to create.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl GpeopleContactGroupCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let group = GpeopleContactGroup {
            name: Some(self.name),
            ..Default::default()
        };
        let created = client.contact_group_create(&group, &[])?.response;

        printer.out(GpeopleContactGroupOutput(created))
    }
}
