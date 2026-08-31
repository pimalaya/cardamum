//! # Card create command
//!
//! Appends a card to an addressbook, from a vCard source, from field
//! flags, from the composer, or from any combination of the three.

use core::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    arg::{AddressbookIdArg, CardComposerArgs},
    card::{
        composer::CardComposer,
        fields::CardFieldsArgs,
        vcard::{CardVersionArg, blank_card, ensure_valid, read_source},
    },
    client::AddressbookClient,
};

/// Append a vCard to the given addressbook.
///
/// The source, the field flags and `-i` stack: the source is the card to
/// start from, the flags set the properties they name on it, and `-i`
/// opens the result in the composer. With no source the card is minted
/// fresh, carrying nothing but a `UID`.
///
/// JSON output: `{"id"}`, the identifier the backend assigned.
#[derive(Debug, Parser)]
pub struct CardCreateCommand {
    /// Addressbook the card is appended to.
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// The composer the card is refined in before it is written.
    #[command(flatten)]
    pub composer: CardComposerArgs,
    /// vCard version a card minted from nothing is written at.
    ///
    /// A card read from a source keeps its own.
    #[arg(long, value_name = "VERSION", default_value = "4.0")]
    pub vcard_version: CardVersionArg,
    /// The properties the command sets on the card.
    #[command(flatten)]
    pub fields: CardFieldsArgs,
    /// vCard to append: a path to a file, raw vCard contents, or `-` for
    /// stdin. Omit to mint one.
    #[arg(value_name = "VCARD")]
    pub vcard: Option<String>,
}

impl CardCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        if self.vcard.is_none() && self.fields.is_empty() && !self.composer.interactive {
            bail!("Nothing to create; give a vCard, a field flag, or -i to compose one");
        }

        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;

        let base = match &self.vcard {
            Some(source) => read_source(source)?,
            None => blank_card(self.vcard_version.into())?,
        };

        let seeded = self.fields.apply(&base)?;

        if !self.composer.interactive {
            // NOTE: only a card cardamum minted is checked. A vCard given
            // on the command line goes to the backend as it was written,
            // which is what the specific commands promise too.
            if self.vcard.is_none() {
                ensure_valid(&seeded)?;
            }

            let id = client.create_card(&addressbook_id, seeded)?;
            return printer.out(CardCreateOutput::Created(CardCreatedOutput { id }));
        }

        let composer = CardComposer {
            command: client.account.card_composer(self.composer.composer)?,
        };

        // NOTE: nothing has reached the network yet, the client opening
        // on the first call that needs it, so the editor runs with no
        // connection held and the create below opens one.
        let Some(draft) = composer.edit(printer, &seeded)? else {
            return printer.out(CardCreateOutput::Abandoned);
        };

        let created = client.create_card(&addressbook_id, draft.contents.clone());
        let id = draft.finish(created)?;

        printer.out(CardCreateOutput::Created(CardCreatedOutput { id }))
    }
}

/// What `card create` prints, which is whether it wrote anything.
///
/// Untagged, so the write serializes exactly as it would on its own. The
/// second shape is reachable through `-i` alone, which `--json` refuses
/// to run.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum CardCreateOutput {
    /// The card the backend created.
    Created(CardCreatedOutput),
    /// Nothing, the edit having been abandoned.
    Abandoned,
}

impl fmt::Display for CardCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created(out) => out.fmt(f),
            Self::Abandoned => writeln!(f, "Card not created"),
        }
    }
}

/// The card the backend created.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardCreatedOutput {
    /// Backend-assigned identifier of the new card.
    ///
    /// On pimdir this is the link id the queued create was staged under,
    /// the store having no id of its own until a sync applies it.
    pub id: String,
}

impl fmt::Display for CardCreatedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Card `{}` successfully created", self.id)
    }
}
