//! # Folder list command
//!
//! Lists the contact folders of the mailbox.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contact_folders::list::MsgraphContactFoldersListParams;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{client::MsgraphClient, contact_folders::render::MsgraphContactFoldersOutput};

/// List the contact folders of the mailbox (one Graph page).
///
/// JSON output: `{"folders": [<raw Graph folder>...]}`.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderListCommand {
    /// Maximum number of folders in the page (`$top`).
    #[arg(short = 's', long, value_name = "N")]
    pub top: Option<u32>,
}

impl MsgraphContactFolderListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let params = MsgraphContactFoldersListParams {
            top: self.top,
            ..Default::default()
        };
        let page = client.contact_folders_list(&params)?.response;

        printer.out(MsgraphContactFoldersOutput {
            folders: page.value,
            next_link: page.next_link,
        })
    }
}
