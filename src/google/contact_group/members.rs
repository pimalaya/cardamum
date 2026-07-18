use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::google::client::GoogleClient;

/// Add and/or remove members of a contact group
/// (`contactGroups.members.modify`). Members are person resource names
/// (`people/<id>`).
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct GoogleContactGroupMembersCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// Person resource name to add (`people/<id>`); repeatable.
    #[arg(long, value_name = "RESOURCE-NAME")]
    pub add: Vec<String>,
    /// Person resource name to remove (`people/<id>`); repeatable.
    #[arg(long, value_name = "RESOURCE-NAME")]
    pub remove: Vec<String>,
}

impl GoogleContactGroupMembersCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        if self.add.is_empty() && self.remove.is_empty() {
            bail!("Pass at least one --add or --remove person resource name");
        }

        let resource_name = format!("contactGroups/{}", self.group_id);
        let modified = client
            .contact_group_members_modify(&resource_name, &self.add, &self.remove)?
            .response;

        if !modified.not_found_resource_names.is_empty() {
            bail!(
                "Some members were not found: {:?}",
                modified.not_found_resource_names
            );
        }

        printer.out(Message::new(format!(
            "Contact group `{}` membership successfully updated (+{} / -{})",
            self.group_id,
            self.add.len(),
            self.remove.len()
        )))
    }
}
