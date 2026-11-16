//! # Contact list command
//!
//! Lists one page of contacts of a folder.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::contacts::{MsgraphContact, list::MsgraphContactsListParams};
use pimalaya_cli::{clap::args::PageTokenArg, output::Paginated, printer::Printer};

use crate::msgraph::{
    client::{MsgraphClient, folder_refusal},
    contacts::render::MsgraphContactsOutput,
};

/// List contacts of the default Contacts folder, or of `--folder` (one
/// Graph page).
///
/// Pass the `nextPage` a page printed to `--page-token` for the next one:
/// the link already carries the folder and the query, so neither is passed
/// again.
///
/// JSON output: `{"contacts": [<raw Graph contact>...], "nextPage"}`,
/// `nextPage` Graph's `@odata.nextLink`, `null` on the last page.
#[derive(Debug, Parser)]
pub struct MsgraphContactListCommand {
    /// Contact folder id; omit for the default Contacts folder.
    #[arg(
        short = 'f',
        long,
        value_name = "FOLDER-ID",
        conflicts_with = "page_token"
    )]
    pub folder: Option<String>,
    /// Maximum number of contacts in the page (`$top`).
    #[arg(
        long,
        short_alias = 's',
        value_name = "N",
        conflicts_with = "page_token"
    )]
    pub top: Option<u32>,
    /// Comma-separated properties to return (`$select`).
    #[arg(long, value_name = "CSV", conflicts_with = "page_token")]
    pub select: Option<String>,
    /// OData filter expression (`$filter`).
    #[arg(long, value_name = "EXPR", conflicts_with = "page_token")]
    pub filter: Option<String>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl MsgraphContactListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let page = match &self.page_token.token {
            Some(link) => client.page_get::<MsgraphContact>(link, None)?,
            None => {
                let params = MsgraphContactsListParams {
                    top: self.top,
                    select: self.select.as_deref(),
                    filter: self.filter.as_deref(),
                    ..Default::default()
                };

                client
                    .contacts_list(self.folder.as_deref(), &params)
                    .map_err(folder_refusal)?
            }
        }
        .response;

        printer.out(Paginated {
            inner: MsgraphContactsOutput {
                contacts: page.value,
            },
            next_page: page.next_link,
        })
    }
}
