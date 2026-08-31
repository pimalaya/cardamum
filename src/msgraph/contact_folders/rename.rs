//! # Folder rename command
//!
//! Patches the display name of a contact folder.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contact_folders::MsgraphContactFolder;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{client::MsgraphClient, contact_folders::render::MsgraphContactFolderOutput};

/// Rename a contact folder (PATCH its `displayName`).
///
/// JSON output: the raw Graph folder object after the update.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderRenameCommand {
    /// Folder id.
    #[arg(value_name = "FOLDER-ID")]
    pub folder_id: String,
    /// New display name.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl MsgraphContactFolderRenameCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let folder = MsgraphContactFolder {
            display_name: self.name,
            ..Default::default()
        };
        let updated = client
            .contact_folder_update(&self.folder_id, &folder)?
            .response;

        printer.out(MsgraphContactFolderOutput(updated))
    }
}
