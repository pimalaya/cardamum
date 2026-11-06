//! # Pimdir count
//!
//! The `pimdir count` command, counting the account's cards a query
//! matches through the store's search index.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::pimdir::{
    backend::PimdirBackend,
    index::{PimdirCardCoverage, parse_query},
};

/// Count the account's cards a query matches.
///
/// Takes the query `pimdir search` takes, over the same cards, and
/// answers from the same index, with what the count could not see.
///
/// JSON output: `{"count", "coverage": {"complete", "bodiless",
/// "unindexed", "behind", "filling"}}`.
#[derive(Debug, Parser)]
pub struct PimdirCountCommand {
    /// Address book to count in, every one of the account's when omitted.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub addressbook: Option<String>,
    /// The query, its words joined by spaces.
    #[arg(value_name = "QUERY", required = true)]
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub query: Vec<String>,
}

impl PimdirCountCommand {
    pub fn execute(self, printer: &mut impl Printer, backend: &PimdirBackend) -> Result<()> {
        let query = parse_query(&self.query, printer.is_json())?;
        let (count, coverage) = backend.count_cards(query, self.addressbook.as_deref())?;
        printer.out(PimdirCountOutput { count, coverage })
    }
}

/// The `pimdir count` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirCountOutput {
    /// How many cards the query matches.
    pub count: u64,
    /// What the count could not see.
    pub coverage: PimdirCardCoverage,
}

impl fmt::Display for PimdirCountOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} card(s)", self.count)?;
        write!(f, "{}", self.coverage)
    }
}
