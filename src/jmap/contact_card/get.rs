//! # ContactCard get command
//!
//! Fetches ContactCards by id.

use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::contact_card::get::JmapContactCardGetOptions;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, render::JmapContactCardsOutput};

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
        let opts = JmapContactCardGetOptions {
            ids: Some(self.ids),
            ..Default::default()
        };
        let out = client.contact_card_get(opts)?;

        printer.out(JmapContactCardsOutput {
            cards: out.cards,
            state: out.new_state,
        })
    }
}
