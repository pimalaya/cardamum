//! # Card build command
//!
//! Builds a vCard from a source, the field flags and the composer, and
//! prints it rather than writing it anywhere.

use core::fmt;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use pimalaya_cli::{
    clap::parsers::path_parser,
    printer::{Message, Printer},
};
use pimalaya_config::command::CommandConfig;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    account::context::Account,
    cli::resolve_account,
    shared::{
        arg::CardComposerArgs,
        card::{
            composer::CardComposer,
            fields::CardFieldsArgs,
            vcard::{CardVersionArg, blank_card, ensure_valid, read_source},
        },
    },
};

/// Build a vCard and print it, writing it nowhere.
///
/// The source, the field flags and `-i` stack as they do on `card
/// create`, and the card comes back on stdout instead of going to a
/// backend, which is how a card is judged before it is sent. No account
/// is resolved unless `-i` needs the configured composer.
///
/// JSON output: `{"contents"}`, the raw vCard as text, or `{"message"}`
/// with `-o`.
#[derive(Debug, Parser)]
pub struct CardBuildCommand {
    /// The composer the card is refined in before it is printed.
    #[command(flatten)]
    pub composer: CardComposerArgs,
    /// Write the card here instead of printing it.
    ///
    /// The composer inherits stdout, so `-i` cannot be redirected: this
    /// is how a card refined in an editor is captured.
    #[arg(short, long, value_name = "PATH", value_parser = path_parser)]
    pub output: Option<PathBuf>,
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
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        let Self {
            composer,
            output,
            vcard_version,
            fields,
            vcard,
        } = self;

        if vcard.is_none() && fields.is_empty() && !composer.interactive {
            bail!("Nothing to build; give a vCard, a field flag, or -i to compose one");
        }

        let base = match &vcard {
            Some(source) => read_source(source)?,
            None => blank_card(vcard_version.into())?,
        };

        let seeded = fields.apply(&base)?;

        if !composer.interactive {
            // NOTE: what a create checks, a build checks too. A built card
            // reaches `card create` as a source, which goes to the backend
            // as it was written, so a laxer check here would be a way
            // around that one.
            if vcard.is_none() {
                ensure_valid(&seeded)?;
            }

            return emit(printer, output.as_deref(), &seeded);
        }

        let composer = CardComposer {
            command: card_composer(printer, config_paths, account_name, composer.composer)?,
        };

        let Some(draft) = composer.edit(printer, &seeded)? else {
            // NOTE: an abandoned edit prints nothing rather than a line
            // saying so: this output is a vCard someone pipes onwards, and
            // a message in that stream is not one.
            return Ok(());
        };

        let emitted = emit(printer, output.as_deref(), &draft.contents);

        draft.finish(emitted)
    }
}

/// Resolves the composer a `-i` build edits in.
///
/// A `--composer` line names it on its own, and only its absence sends
/// the command looking for a configuration: building a card is otherwise
/// something a machine holding none can do.
fn card_composer(
    printer: &mut impl Printer,
    config_paths: &[PathBuf],
    account_name: Option<&str>,
    flag: Option<String>,
) -> Result<CommandConfig> {
    if let Some(line) = flag {
        return Ok(CommandConfig::Shell(line));
    }

    let (config, _name, account_config) = resolve_account(printer, config_paths, account_name)?;
    let account = Account::from(config).merge(Account::from(account_config));

    account.card_composer(None)
}

/// Hands the card over: to the file `-o` names, or to the printer.
fn emit(printer: &mut impl Printer, output: Option<&Path>, card: &[u8]) -> Result<()> {
    let Some(path) = output else {
        return printer.out(CardBuildOutput {
            contents: String::from_utf8(card.to_vec())?,
        });
    };

    fs::write(path, card).with_context(|| format!("Cannot write vCard {path:?}"))?;

    printer.out(Message::new(format!(
        "Card successfully written to {}",
        path.display()
    )))
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
