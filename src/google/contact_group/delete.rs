use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::google::client::GoogleClient;

/// Delete a contact group. Its member contacts stay in `myContacts`
/// unless `--delete-contacts` is set.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct GoogleContactGroupDeleteCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// Also delete the member contacts, not just the group.
    #[arg(long)]
    pub delete_contacts: bool,
}

impl GoogleContactGroupDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let resource_name = format!("contactGroups/{}", self.group_id);
        client.contact_group_delete(&resource_name, self.delete_contacts)?;

        printer.out(Message::new(format!(
            "Contact group `{}` successfully deleted",
            self.group_id
        )))
    }
}
