//! # Contact group get command
//!
//! Reads one contact group by id (`contactGroups.get`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::rest::contact_groups::GpeopleGroupField;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, render::GpeopleContactGroupOutput};

/// GET a contact group by id.
///
/// JSON output: the raw People group object.
#[derive(Debug, Parser)]
pub struct GpeopleContactGroupGetCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// Maximum number of member resource names to include.
    #[arg(short = 'm', long, value_name = "N")]
    pub max_members: Option<u32>,
}

impl GpeopleContactGroupGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let resource_name = format!("contactGroups/{}", self.group_id);
        // NOTE: People returns the member count only when the mask asks
        // for it.
        let fields = [
            GpeopleGroupField::Name,
            GpeopleGroupField::GroupType,
            GpeopleGroupField::MemberCount,
        ];
        let group = client
            .contact_group_get(&resource_name, self.max_members, &fields)?
            .response;

        printer.out(GpeopleContactGroupOutput(group))
    }
}
