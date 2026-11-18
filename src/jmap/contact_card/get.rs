//! # ContactCard get command
//!
//! Fetches ContactCards by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, raw, render::JmapContactCardsOutput};

/// Get JMAP ContactCards by id (`ContactCard/get`).
///
/// JSON output: `{"list": [<ContactCard as the server sent it>...],
/// "state"}`.
#[derive(Debug, Parser)]
pub struct JmapContactCardGetCommand {
    /// ContactCard ids to fetch.
    #[arg(value_name = "CARD-ID", required = true)]
    pub ids: Vec<String>,
}

impl JmapContactCardGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let out = raw::get(&mut client, "ContactCard", Some(self.ids))?;

        printer.out(JmapContactCardsOutput {
            cards: out.list,
            state: Some(out.state),
            query_state: None,
        })
    }
}
