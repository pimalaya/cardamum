//! # Contact group get command
//!
//! Reads one contact group by id (`contactGroups.get`).

use anyhow::Result;
use clap::Parser;
use io_people::v1::rest::contact_groups::PeopleGroupField;
use pimalaya_cli::printer::Printer;

use crate::people::{client::PeopleClient, render::PeopleContactGroupOutput};

/// GET a contact group by id.
///
/// JSON output: the raw People group object.
#[derive(Debug, Parser)]
pub struct PeopleContactGroupGetCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// Maximum number of member resource names to include.
    #[arg(short = 'm', long, value_name = "N")]
    pub max_members: Option<u32>,
}

impl PeopleContactGroupGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: PeopleClient) -> Result<()> {
        let resource_name = format!("contactGroups/{}", self.group_id);
        // NOTE: People returns the member count only when the mask asks
        // for it.
        let fields = [
            PeopleGroupField::Name,
            PeopleGroupField::GroupType,
            PeopleGroupField::MemberCount,
        ];
        let group = client
            .contact_group_get(&resource_name, self.max_members, &fields)?
            .response;

        printer.out(PeopleContactGroupOutput(group))
    }
}
