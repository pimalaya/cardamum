//! # Connection list command
//!
//! Lists one page of the signed-in user's contacts
//! (`people.connections.list`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::rest::people::{
    connections::list::GpeopleConnectionsListParams, vcard::GPEOPLE_PERSON_VCARD_FIELDS,
};
use pimalaya_cli::{clap::args::PageTokenArg, output::Paginated, printer::Printer};

use crate::gpeople::{client::GpeopleClient, render::GpeoplePersonsOutput};

/// List the signed-in user's contacts (one People page).
///
/// Pass a `--sync-token` from a previous list for incremental sync, and
/// the `nextPage` a page printed to `--page-token` for the next one, with
/// the same other flags.
///
/// JSON output: `{"people": [<raw People person>...], "nextSyncToken",
/// "nextPage"}`, `nextPage` People's `nextPageToken`, `null` on the last
/// page.
#[derive(Debug, Parser)]
pub struct GpeopleConnectionListCommand {
    /// Maximum number of contacts in the page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
    /// Sync token from a previous list, for incremental sync.
    #[arg(long, value_name = "TOKEN")]
    pub sync_token: Option<String>,
}

impl GpeopleConnectionListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let params = GpeopleConnectionsListParams {
            page_size: self.page_size,
            page_token: self.page_token.token.as_deref(),
            sync_token: self.sync_token.as_deref(),
            request_sync_token: true,
            ..Default::default()
        };
        let page = client
            .connections_list(GPEOPLE_PERSON_VCARD_FIELDS, &params)?
            .response;

        printer.out(Paginated {
            inner: GpeoplePersonsOutput {
                people: page.connections,
                next_sync_token: page.next_sync_token,
            },
            next_page: page.next_page_token,
        })
    }
}
