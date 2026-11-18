//! # Folder get command
//!
//! Reads a single contact folder by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, folder_refusal},
    contact_folders::render::MsgraphContactFolderGetOutput,
};

/// GET a contact folder by id.
///
/// JSON output: the folder as Graph sent it, every property kept.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderGetCommand {
    /// Folder id.
    #[arg(value_name = "FOLDER-ID")]
    pub folder_id: String,
}

impl MsgraphContactFolderGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let folder = client
            .raw_get(&["contactFolders", &self.folder_id])
            .map_err(folder_refusal)?;
        printer.out(MsgraphContactFolderGetOutput(folder))
    }
}
