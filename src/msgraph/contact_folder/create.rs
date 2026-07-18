use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contact_folders::MsgraphContactFolder;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{client::MsgraphClient, contact_folder::render::FolderReport};

/// Create a contact folder.
///
/// JSON output: the raw Graph folder object the server created.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderCreateCommand {
    /// Display name of the folder to create.
    #[arg(value_name = "NAME")]
    pub name: String,
}

impl MsgraphContactFolderCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let folder = MsgraphContactFolder {
            display_name: self.name,
            ..Default::default()
        };
        let created = client.contact_folder_create(&folder)?.response;

        printer.out(FolderReport(created))
    }
}
