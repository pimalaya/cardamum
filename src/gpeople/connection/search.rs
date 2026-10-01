//! # Connection search command
//!
//! Searches the signed-in user's contacts by query string
//! (`people.searchContacts`).

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, project, render::GpeoplePersonsOutput};

/// Search the signed-in user's contacts by query string.
///
/// JSON output: `{"people": [<raw People person>...]}`.
#[derive(Debug, Parser)]
pub struct GpeopleConnectionSearchCommand {
    /// Query string matched against names, nicknames, emails, phones.
    #[arg(value_name = "QUERY")]
    pub query: String,
    /// Maximum number of results.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
}

impl GpeopleConnectionSearchCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
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

        printer.out(GpeoplePersonsOutput {
            preset,
            id_color,
            people,
            next_page_token: None,
            next_sync_token: None,
        })
    }
}
