use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::google::{client::GoogleClient, project, render::PersonsReport};

/// Search the signed-in user's contacts by query string
/// (`people.searchContacts`).
///
/// JSON output: `{"people": [<raw People person>...]}`.
#[derive(Debug, Parser)]
pub struct GoogleConnectionSearchCommand {
    /// Query string matched against names, nicknames, emails, phones.
    #[arg(value_name = "QUERY")]
    pub query: String,
    /// Maximum number of results.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
}

impl GoogleConnectionSearchCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GoogleClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();

        let response = client
            .contacts_search(&self.query, project::READ_FIELDS, self.page_size, &[])?
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
