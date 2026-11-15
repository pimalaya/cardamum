//! # Folder get command
//!
//! Reads a single contact folder by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, folder_refusal},
    contact_folders::render::MsgraphContactFolderOutput,
};

/// GET a contact folder by id.
///
/// JSON output: the raw Graph folder object.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderGetCommand {
    /// Folder id.
    #[arg(value_name = "FOLDER-ID")]
    pub folder_id: String,
}

impl MsgraphContactFolderGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let folder = client
            .contact_folder_get(&self.folder_id)
            .map_err(folder_refusal)?
            .response;
        printer.out(MsgraphContactFolderOutput(folder))
    }
}
