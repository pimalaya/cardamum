use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::contact_card::get::JmapContactCardGetOptions;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, render::CardsReport};

/// Get JMAP ContactCards by id (`ContactCard/get`).
///
/// JSON output: `{"list": [<raw JMAP ContactCard>...], "state"}`.
#[derive(Debug, Parser)]
pub struct JmapContactCardGetCommand {
    /// ContactCard ids to fetch.
    #[arg(value_name = "CARD-ID", required = true)]
    pub ids: Vec<String>,
}

impl JmapContactCardGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let id_color = client.account.cards_list_table_id_color();

        let opts = JmapContactCardGetOptions {
            ids: Some(self.ids),
            ..Default::default()
        };
        let out = client.contact_card_get(opts)?;

        printer.out(CardsReport {
            preset,
            id_color,
            cards: out.cards,
            state: out.new_state,
        })
    }
}
