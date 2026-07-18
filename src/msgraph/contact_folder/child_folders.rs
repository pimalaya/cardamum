use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contact_folders::list::MsgraphContactFoldersListParams;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{client::MsgraphClient, contact_folder::render::FoldersReport};

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
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.addressbooks_list_table_id_color();

        let params = MsgraphContactFoldersListParams {
            top: self.top,
            ..Default::default()
        };
        let page = client
            .contact_child_folders_list(&self.folder_id, &params)?
            .response;

        printer.out(FoldersReport {
            preset,
            id_color,
            folders: page.value,
            next_link: page.next_link,
        })
    }
}
