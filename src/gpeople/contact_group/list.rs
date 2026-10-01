//! # Contact group list command
//!
//! Lists one page of contact groups (`contactGroups.list`).

use anyhow::Result;
use clap::Parser;
use io_gpeople::v1::rest::contact_groups::{
    GpeopleGroupField, list::GpeopleContactGroupsListParams,
};
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, render::GpeopleContactGroupsOutput};

/// List the contact groups (one People page).
///
/// JSON output: `{"contactGroups": [<raw People group>...]}`.
#[derive(Debug, Parser)]
pub struct GpeopleContactGroupListCommand {
    /// Maximum number of groups in the page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
    /// Sync token from a previous list, for incremental sync.
    #[arg(long, value_name = "TOKEN")]
    pub sync_token: Option<String>,
}

impl GpeopleContactGroupListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.addressbooks_list_table_id_color();

        let params = GpeopleContactGroupsListParams {
            page_size: self.page_size,
            sync_token: self.sync_token.as_deref(),
            ..Default::default()
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

        printer.out(GpeopleContactGroupsOutput {
            preset,
            id_color,
            groups: page.contact_groups,
            next_page_token: page.next_page_token,
            next_sync_token: page.next_sync_token,
        })
    }
}
