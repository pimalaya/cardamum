//! # ContactCard get command
//!
//! Fetches ContactCards by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    error::{ErrorCode, partial_when},
    jmap::{client::JmapClient, raw, render::JmapContactCardsOutput},
};

/// Get JMAP ContactCards by id (`ContactCard/get`).
///
/// JSON output: `{"list": [<ContactCard as the server sent it>...],
/// "state", "errors"}`, `errors` holding a `card-not-found` error for
/// each id the server found nothing for, `[]` when none, the command then
/// exiting 3.
#[derive(Debug, Parser)]
pub struct JmapContactCardGetCommand {
    /// ContactCard ids to fetch.
    #[arg(value_name = "CARD-ID", required = true)]
    pub ids: Vec<String>,
}

impl JmapContactCardGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let out = raw::get(&mut client, "ContactCard", Some(self.ids))?;
        let errors = out.not_found_errors(ErrorCode::CardNotFound, "ContactCard");

        printer.out(JmapContactCardsOutput {
            cards: out.list,
            state: Some(out.state),
            query_state: None,
            errors: Some(errors.clone()),
        })?;

        partial_when(&errors)
    }
}
