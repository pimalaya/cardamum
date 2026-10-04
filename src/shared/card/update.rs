//! # Card update command
//!
//! Rewrites an existing card, from a vCard source, from field flags, from
//! the composer, or from any combination of the three, and reports the
//! properties the server would not let go.

use core::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    arg::{AddressbookIdArg, CardComposerArgs},
    card::{composer::CardComposer, fields::CardFieldsArgs, vcard::read_source},
    client::AddressbookClient,
};

/// Rewrite an existing vCard.
///
/// The source, the field flags and `-i` stack: the source replaces the
/// card's bytes, the flags set the properties they name on it, and `-i`
/// opens the result in the composer. With no source the card is read from
/// the backend first, and the version it answered guards the write.
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
    /// 9110 `If-Match`), which is how a concurrent write is caught. With
    /// no source the command sends the ETag it read when this is omitted.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
    /// The composer the card is refined in before it is written.
    #[command(flatten)]
    pub composer: CardComposerArgs,
    /// The properties the command sets on the card.
    #[command(flatten)]
    pub fields: CardFieldsArgs,
    /// Card to update, as `card list` reports it.
    ///
    /// This is the backend's own id, not the vCard `UID`, which names no
    /// card on its own.
    #[arg(value_name = "CARD-ID")]
    pub card_id: String,
    /// vCard replacing the current contents: a path to a file, raw vCard
    /// contents, or `-` for stdin. Omit to start from the stored card.
    #[arg(value_name = "VCARD")]
    pub vcard: Option<String>,
}

impl CardUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        if self.vcard.is_none() && self.fields.is_empty() && !self.composer.interactive {
            bail!("Nothing to update; give a vCard, a field flag, or -i to edit the card");
        }

        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;

        // NOTE: reading the card to start from also gives the version it
        // is being changed from, so an edit that took a minute cannot
        // silently overwrite a write that landed during it.
        let (base, etag) = match &self.vcard {
            Some(source) => (read_source(source)?, None),
            None => {
                let card = client.get_card(&addressbook_id, &self.card_id)?;
                (card.contents, card.etag)
            }
        };

        let if_match = self.if_match.or(etag);
        let seeded = self.fields.apply(&base)?;

        if !self.composer.interactive {
            let outcome =
                client.update_card(&addressbook_id, &self.card_id, seeded, if_match.as_deref())?;

            return printer.out(CardUpdateOutput::Applied(CardUpdatedOutput {
                id: self.card_id,
                kept_properties: outcome.kept_properties,
            }));
        }

        let composer = CardComposer {
            command: client.account.card_composer(self.composer.composer)?,
        };

        // NOTE: reading the card is the one thing that has to happen
        // before the editor, so its connection is dropped here rather than
        // left idle for the minutes an edit takes: a server closes such a
        // connection, and the update below opens a fresh one.
        client.disconnect();

        let Some(draft) = composer.edit(printer, &seeded)? else {
            return printer.out(CardUpdateOutput::Abandoned);
        };

        let updated = client.update_card(
            &addressbook_id,
            &self.card_id,
            draft.contents.clone(),
            if_match.as_deref(),
        );

        let outcome = draft.finish(updated)?;

        printer.out(CardUpdateOutput::Applied(CardUpdatedOutput {
            id: self.card_id,
            kept_properties: outcome.kept_properties,
        }))
    }
}

/// What `card update` prints, which is whether it wrote anything.
///
/// Untagged, so the write serializes exactly as it would on its own. The
/// second shape is reachable through `-i` alone, which `--json` refuses
/// to run.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum CardUpdateOutput {
    /// The card the backend updated.
    Applied(CardUpdatedOutput),
    /// Nothing, the edit having been abandoned.
    Abandoned,
}

impl fmt::Display for CardUpdateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Applied(out) => out.fmt(f),
            Self::Abandoned => writeln!(f, "Card not updated"),
        }
    }
}

/// The card the backend updated, and what it would not let go.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardUpdatedOutput {
    /// Backend-specific identifier of the updated card.
    pub id: String,
    /// Names of the vCard properties the update dropped and the server
    /// kept anyway, empty on a backend that can clear everything.
    pub kept_properties: Vec<String>,
}

impl fmt::Display for CardUpdatedOutput {
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
