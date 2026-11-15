//! # Contact group list command
//!
//! Lists one page of contact groups (`contactGroups.list`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::rest::contact_groups::{
    GpeopleGroupField, list::GpeopleContactGroupsListParams,
};
use pimalaya_cli::{clap::args::PageTokenArg, output::Paginated, printer::Printer};

use crate::gpeople::{client::GpeopleClient, render::GpeopleContactGroupsOutput};

/// List the contact groups (one People page).
///
/// Pass the `nextPage` a page printed to `--page-token` for the next one,
/// with the same other flags.
///
/// JSON output: `{"contactGroups": [<raw People group>...],
/// "nextSyncToken", "nextPage"}`, `nextPage` People's `nextPageToken`,
/// `null` on the last page.
#[derive(Debug, Parser)]
pub struct GpeopleContactGroupListCommand {
    /// Maximum number of groups in the page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
    /// Sync token from a previous list, for incremental sync.
    #[arg(long, value_name = "TOKEN")]
    pub sync_token: Option<String>,
}

impl GpeopleContactGroupListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let params = GpeopleContactGroupsListParams {
            page_size: self.page_size,
            page_token: self.page_token.token.as_deref(),
            sync_token: self.sync_token.as_deref(),
        };
        // NOTE: People returns a group's member count only when the mask
        // asks for it, and an absent count renders as 0, which reads as
        // an empty group rather than as a question never asked.
        let fields = [
            GpeopleGroupField::Name,
            GpeopleGroupField::GroupType,
            GpeopleGroupField::MemberCount,
        ];
        let page = client.contact_groups_list(&fields, &params)?.response;

        printer.out(Paginated {
            inner: GpeopleContactGroupsOutput {
                groups: page.contact_groups,
                next_sync_token: page.next_sync_token,
            },
            next_page: page.next_page_token,
        })
    }
}
