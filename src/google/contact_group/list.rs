use anyhow::Result;
use clap::Parser;
use io_people::v1::rest::contact_groups::list::PeopleContactGroupsListParams;
use pimalaya_cli::printer::Printer;

use crate::google::{client::GoogleClient, render::GroupsReport};

/// List the contact groups (one People page).
///
/// JSON output: `{"contactGroups": [<raw People group>...]}`.
#[derive(Debug, Parser)]
pub struct GoogleContactGroupListCommand {
    /// Maximum number of groups in the page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
    /// Sync token from a previous list, for incremental sync.
    #[arg(long, value_name = "TOKEN")]
    pub sync_token: Option<String>,
}

impl GoogleContactGroupListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.addressbooks_list_table_id_color();

        let params = PeopleContactGroupsListParams {
            page_size: self.page_size,
            sync_token: self.sync_token.as_deref(),
            ..Default::default()
        };
        let page = client.contact_groups_list(&[], &params)?.response;

        printer.out(GroupsReport {
            preset,
            id_color,
            groups: page.contact_groups,
            next_page_token: page.next_page_token,
            next_sync_token: page.next_sync_token,
        })
    }
}
