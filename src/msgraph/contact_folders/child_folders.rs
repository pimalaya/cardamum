//! # Child folder list command
//!
//! Lists the folders nested under a contact folder.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contact_folders::{
    MsgraphContactFolder, list::MsgraphContactFoldersListParams,
};
use pimalaya_cli::{clap::args::PageTokenArg, output::Paginated, printer::Printer};

use crate::msgraph::{
    client::{MsgraphClient, folder_refusal},
    contact_folders::render::MsgraphContactFoldersOutput,
};

/// List the child folders of a contact folder (one Graph page).
///
/// Pass the `nextPage` a page printed to `--page-token` for the next one:
/// the link already carries the parent folder and the page size, so the
/// folder id may be left out.
///
/// JSON output: `{"folders": [<raw Graph folder>...], "nextPage"}`,
/// `nextPage` Graph's `@odata.nextLink`, `null` on the last page.
#[derive(Debug, Parser)]
pub struct MsgraphContactFolderChildFoldersCommand {
    /// Parent folder id.
    #[arg(value_name = "FOLDER-ID", required_unless_present = "page_token")]
    pub folder_id: Option<String>,
    /// Maximum number of folders in the page (`$top`).
    #[arg(
        long,
        short_alias = 's',
        value_name = "N",
        conflicts_with = "page_token"
    )]
    pub top: Option<u32>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl MsgraphContactFolderChildFoldersCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        // NOTE: clap requires the folder id when no page token is given.
        let folder_id = self.folder_id.unwrap_or_default();
        let page = match &self.page_token.token {
            Some(link) => client.page_get::<MsgraphContactFolder>(link, None)?,
            None => {
                let params = MsgraphContactFoldersListParams {
                    top: self.top,
                    ..Default::default()
                };

                client
                    .contact_child_folders_list(&folder_id, &params)
                    .map_err(folder_refusal)?
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
