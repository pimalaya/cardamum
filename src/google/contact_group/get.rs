use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::google::{client::GoogleClient, render::GroupReport};

/// GET a contact group by id.
///
/// JSON output: the raw People group object.
#[derive(Debug, Parser)]
pub struct GoogleContactGroupGetCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// Maximum number of member resource names to include.
    #[arg(short = 'm', long, value_name = "N")]
    pub max_members: Option<u32>,
}

impl GoogleContactGroupGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let resource_name = format!("contactGroups/{}", self.group_id);
        let group = client
            .contact_group_get(&resource_name, self.max_members, &[])?
            .response;

        printer.out(GroupReport(group))
    }
}
