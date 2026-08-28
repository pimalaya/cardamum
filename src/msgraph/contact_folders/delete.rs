use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::msgraph::client::MsgraphClient;

/// Delete a contact folder and every contact it contains.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderDeleteCommand {
    /// Folder id.
    #[arg(value_name = "FOLDER-ID")]
    pub folder_id: String,
}

impl MsgraphContactFolderDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        client.contact_folder_delete(&self.folder_id)?;

        printer.out(Message::new(format!(
            "Contact folder `{}` successfully deleted",
            self.folder_id
        )))
    }
}
