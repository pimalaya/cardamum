//! # Folder delete command
//!
//! Deletes a contact folder and everything it holds.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{msgraph::client::MsgraphClient, shared::receipt::IdReceipt};

/// Delete a contact folder and every contact it contains.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderDeleteCommand {
    /// Folder id.
    #[arg(value_name = "FOLDER-ID")]
    pub folder_id: String,
}

impl MsgraphContactFolderDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        client.contact_folder_delete(&self.folder_id)?;

        printer.out(IdReceipt {
            id: self.folder_id,
            done: "Contact folder deleted",
        })
    }
}
