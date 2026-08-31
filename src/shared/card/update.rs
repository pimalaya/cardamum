//! # Card update command
//!
//! Replaces the vCard of an existing card, and reports the properties the
//! server would not let go.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{arg::AddressbookIdArg, card::vcard::VcardArg, client::AddressbookClient};

/// Replace the bytes of an existing vCard.
///
/// JSON output: `{"id", "keptProperties"}`, the second listing what the
/// update dropped and the server kept anyway.
#[derive(Debug, Parser)]
pub struct CardUpdateCommand {
    /// Addressbook holding the card.
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// Gate the update on this ETag, as returned by a previous read.
    ///
    /// The update only lands if the server still holds that version (RFC
    /// 9110 `If-Match`), which is how a concurrent write is caught.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
    /// Card to update, as `card list` reports it.
    ///
    /// This is the backend's own id, not the vCard `UID`, which names no
    /// card on its own.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
    /// vCard replacing the current contents.
    #[command(flatten)]
    pub vcard: VcardArg,
}

impl CardUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let contents = self.vcard.read()?;

        let outcome = client.update_card(
            &addressbook_id,
            &self.card_id,
            contents,
            self.if_match.as_deref(),
        )?;

        printer.out(CardUpdateOutput {
            id: self.card_id,
            kept_properties: outcome.kept_properties,
        })
    }
}

/// The card the backend updated, and what it would not let go.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardUpdateOutput {
    /// Backend-specific identifier of the updated card.
    pub id: String,
    /// Names of the vCard properties the update dropped and the server
    /// kept anyway, empty on a backend that can clear everything.
    pub kept_properties: Vec<String>,
}

impl fmt::Display for CardUpdateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Card `{}` successfully updated", self.id)?;

        // NOTE: the write landed, but not all of it: saying so beats a
        // clean success for a card that kept what the vCard dropped.
        if !self.kept_properties.is_empty() {
            write!(
                f,
                ", except {}, which the server will not let go",
                self.kept_properties.join(", ")
            )?;
        }

        writeln!(f)
    }
}
