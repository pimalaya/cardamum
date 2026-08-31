//! # Card build command
//!
//! Builds a vCard from a source and the field flags, and prints it
//! rather than writing it anywhere.

use core::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::card::{
    fields::CardFieldsArgs,
    vcard::{CardVersionArg, blank_card, ensure_valid, read_source},
};

/// Build a vCard and print it, writing it nowhere.
///
/// The source and the field flags stack as they do on `card create`: the
/// source is the card to start from, and the flags set the properties
/// they name on it. No backend is reached and no account is resolved, so
/// this runs on a machine holding no configuration at all.
///
/// JSON output: `{"contents"}`, the raw vCard as text.
#[derive(Debug, Parser)]
pub struct CardBuildCommand {
    /// vCard version a card built from nothing is written at.
    ///
    /// A card read from a source keeps its own.
    #[arg(long, value_name = "VERSION", default_value = "4.0")]
    pub vcard_version: CardVersionArg,
    /// The properties the command sets on the card.
    #[command(flatten)]
    pub fields: CardFieldsArgs,
    /// vCard to build on: a path to a file, raw vCard contents, or `-`
    /// for stdin. Omit to mint one.
    #[arg(value_name = "VCARD")]
    pub vcard: Option<String>,
}

impl CardBuildCommand {
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        if self.vcard.is_none() && self.fields.is_empty() {
            bail!("Nothing to build; give a vCard or a field flag");
        }

        let base = match &self.vcard {
            Some(source) => read_source(source)?,
            None => blank_card(self.vcard_version.into())?,
        };

        let built = self.fields.apply(&base)?;

        // NOTE: what a create checks, a build checks too. A built card
        // reaches `card create` as a source, which goes to the backend as
        // it was written, so a laxer check here would be a way around
        // that one.
        if self.vcard.is_none() {
            ensure_valid(&built)?;
        }

        printer.out(CardBuildOutput {
            contents: String::from_utf8(built)?,
        })
    }
}

/// The vCard the command built.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardBuildOutput {
    /// The raw vCard, as text.
    pub contents: String,
}

impl fmt::Display for CardBuildOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents)
    }
}
