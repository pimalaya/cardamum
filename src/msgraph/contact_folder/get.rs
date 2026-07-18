use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{client::MsgraphClient, contact_folder::render::FolderReport};

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
        let folder = client.contact_folder_get(&self.folder_id)?.response;
        printer.out(FolderReport(folder))
    }
}
