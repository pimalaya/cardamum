use anyhow::Result;
use clap::Parser;
use io_people::v1::rest::other_contacts::list::PeopleOtherContactsListParams;
use pimalaya_cli::printer::Printer;

use crate::google::{
    client::GoogleClient, other_contact::fields::OTHER_CONTACT_FIELDS, render::PersonsReport,
};

/// List the "other contacts" (one People page). Pass a `--sync-token`
/// from a previous list for incremental sync.
///
/// JSON output: `{"people": [<raw People person>...]}`.
#[derive(Debug, Parser)]
pub struct GoogleOtherContactListCommand {
    /// Maximum number of contacts in the page.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
    /// Sync token from a previous list, for incremental sync.
    #[arg(long, value_name = "TOKEN")]
    pub sync_token: Option<String>,
}

impl GoogleOtherContactListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();

        let params = PeopleOtherContactsListParams {
            page_size: self.page_size,
            sync_token: self.sync_token.as_deref(),
            request_sync_token: true,
            ..Default::default()
        };
        let page = client
            .other_contacts_list(OTHER_CONTACT_FIELDS, &params)?
            .response;

        printer.out(PersonsReport {
            preset,
            id_color,
            people: page.other_contacts,
            next_page_token: page.next_page_token,
            next_sync_token: page.next_sync_token,
        })
    }
}
