//! # Folder list command
//!
//! Lists the contact folders of the mailbox.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contact_folders::{
    MsgraphContactFolder, list::MsgraphContactFoldersListParams,
};
use pimalaya_cli::{clap::args::PageTokenArg, output::Paginated, printer::Printer};

use crate::msgraph::{client::MsgraphClient, contact_folders::render::MsgraphContactFoldersOutput};

/// List the contact folders of the mailbox (one Graph page).
///
/// Pass the `nextPage` a page printed to `--page-token` for the next one:
/// the link already carries the page size.
///
/// JSON output: `{"folders": [<raw Graph folder>...], "nextPage"}`,
/// `nextPage` Graph's `@odata.nextLink`, `null` on the last page.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderListCommand {
    /// Maximum number of folders in the page (`$top`).
    #[arg(short = 's', long, value_name = "N", conflicts_with = "page_token")]
    pub top: Option<u32>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl MsgraphContactFolderListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let page = match &self.page_token.token {
            Some(link) => client.page_get::<MsgraphContactFolder>(link, None)?,
            None => {
                let params = MsgraphContactFoldersListParams {
                    top: self.top,
                    ..Default::default()
                };

                client.contact_folders_list(&params)?
            }
        }
        .response;

        printer.out(Paginated {
            inner: MsgraphContactFoldersOutput {
                folders: page.value,
            },
            next_page: page.next_link,
        })
    }
}
