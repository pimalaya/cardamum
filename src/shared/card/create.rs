//! # Card create command
//!
//! Reads a vCard from a file or from stdin and appends it to an
//! addressbook.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{arg::AddressbookIdArg, card::vcard::VcardArg, client::AddressbookClient};

/// Append a vCard to the given addressbook.
///
/// JSON output: `{"id"}`, the identifier the backend assigned.
#[derive(Debug, Parser)]
pub struct CardCreateCommand {
    /// Addressbook the card is appended to.
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// vCard to append.
    #[command(flatten)]
    pub vcard: VcardArg,
}

impl CardCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let contents = self.vcard.read()?;
        let id = client.create_card(&addressbook_id, contents)?;

        printer.out(CardCreateOutput { id })
    }
}

/// The card the backend created.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardCreateOutput {
    /// Backend-assigned identifier of the new card.
    ///
    /// On pimdir this is the link id the queued create was staged under,
    /// the store having no id of its own until a sync applies it.
    pub id: String,
}

impl fmt::Display for CardCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Card `{}` successfully created", self.id)
    }
}
