//! # Contact group get command
//!
//! Reads one contact group by id (`contactGroups.get`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::{query::to_field_mask, rest::contact_groups::GpeopleGroupField};
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, render::GpeopleContactGroupGetOutput};

/// GET a contact group by id.
///
/// JSON output: the group as People sent it, every property kept.
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
        // NOTE: People returns the member count only when the mask asks
        // for it.
        let fields = [
            GpeopleGroupField::Name,
            GpeopleGroupField::GroupType,
            GpeopleGroupField::MemberCount,
        ];
        let mut pairs = vec![("groupFields", to_field_mask(&fields))];
        if let Some(max) = self.max_members {
            pairs.push(("maxMembers", max.to_string()));
        }
        let group = client.raw_get("contactGroups", &self.group_id, &pairs)?;

        printer.out(GpeopleContactGroupGetOutput(group))
    }
}
