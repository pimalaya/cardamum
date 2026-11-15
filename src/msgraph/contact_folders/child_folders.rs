//! # Child folder list command
//!
//! Lists the folders nested under a contact folder.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contact_folders::list::MsgraphContactFoldersListParams;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, folder_refusal},
    contact_folders::render::MsgraphContactFoldersOutput,
};

/// List the child folders of a contact folder (one Graph page).
///
/// JSON output: `{"folders": [<raw Graph folder>...]}`.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderChildFoldersCommand {
    /// Parent folder id.
    #[arg(value_name = "FOLDER-ID")]
    pub folder_id: String,
    /// Maximum number of folders in the page (`$top`).
    #[arg(short = 's', long, value_name = "N")]
    pub top: Option<u32>,
}

impl MsgraphContactFolderChildFoldersCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let params = MsgraphContactFoldersListParams {
            top: self.top,
            ..Default::default()
        };
        let page = client
            .contact_child_folders_list(&self.folder_id, &params)
            .map_err(folder_refusal)?
            .response;

        printer.out(MsgraphContactFoldersOutput {
            folders: page.value,
            next_link: page.next_link,
        })
    }
}
