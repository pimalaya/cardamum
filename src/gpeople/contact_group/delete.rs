//! # Contact group delete command
//!
//! Deletes a contact group (`contactGroups.delete`).

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{gpeople::client::GpeopleClient, shared::receipt::IdReceipt};

/// Delete a contact group.
///
/// Its member contacts stay in `myContacts` unless `--delete-contacts`
/// is set.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct GpeopleContactGroupDeleteCommand {
    /// Group id (the segment after `contactGroups/`).
    #[arg(value_name = "GROUP-ID")]
    pub group_id: String,
    /// Also delete the member contacts, not just the group.
    #[arg(long)]
    pub delete_contacts: bool,
}

impl GpeopleContactGroupDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let resource_name = format!("contactGroups/{}", self.group_id);
        client.contact_group_delete(&resource_name, self.delete_contacts)?;

        printer.out(IdReceipt {
            id: self.group_id,
            done: "Contact group deleted",
        })
    }
}
