use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::google::{
    client::GoogleClient, other_contact::fields::OTHER_CONTACT_FIELDS, render::PersonsReport,
};

/// Search the "other contacts" by query string
/// (`otherContacts.search`).
///
/// JSON output: `{"people": [<raw People person>...]}`.
#[derive(Debug, Parser)]
pub struct GoogleOtherContactSearchCommand {
    /// Query string matched against names, emails and phones.
    #[arg(value_name = "QUERY")]
    pub query: String,
    /// Maximum number of results.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
}

impl GoogleOtherContactSearchCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();

        let response = client
            .other_contacts_search(&self.query, OTHER_CONTACT_FIELDS, self.page_size)?
            .response;
        let people = response
            .results
            .into_iter()
            .filter_map(|result| result.person)
            .collect();

        printer.out(PersonsReport {
            preset,
            id_color,
            people,
            next_page_token: None,
            next_sync_token: None,
        })
    }
}
