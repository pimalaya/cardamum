use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::contact_card::query::{JmapContactCardFilter, JmapContactCardQueryOptions};
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, render::CardsReport};

/// Query JMAP ContactCards (`ContactCard/query` + `/get`).
///
/// JSON output: `{"list": [<raw JMAP ContactCard>...], "state"}`.
#[derive(Debug, Parser)]
pub struct JmapContactCardQueryCommand {
    /// Restrict to cards in this AddressBook.
    #[arg(short = 'k', long, value_name = "ADDRESS-BOOK")]
    pub in_address_book: Option<String>,
    /// Free-text match against any text in the card.
    #[arg(long, value_name = "TEXT")]
    pub text: Option<String>,
    /// Maximum number of cards to return.
    #[arg(short = 'l', long, value_name = "N")]
    pub limit: Option<u64>,
}

impl JmapContactCardQueryCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();

        let filter = JmapContactCardFilter {
            in_address_book: self.in_address_book,
            text: self.text,
            ..Default::default()
        };
        let opts = JmapContactCardQueryOptions {
            filter: Some(filter),
            limit: self.limit,
            ..Default::default()
        };
        let out = client.contact_card_query(opts)?;

        printer.out(CardsReport {
            preset,
            id_color,
            cards: out.cards,
            state: out.query_state,
        })
    }
}
