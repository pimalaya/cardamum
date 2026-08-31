//! # Shared arguments
//!
//! Clap arguments reused across the shared API commands.

use clap::Parser;

/// Names the addressbook a shared API command operates on.
#[derive(Debug, Parser)]
pub struct AddressbookIdArg {
    /// Addressbook the command operates on.
    ///
    /// Falls back to the `addressbook.default` config when omitted,
    /// otherwise the command bails.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub id: Option<String>,
}

/// The composer `card create` and `card update` refine a card in.
#[derive(Debug, Parser)]
pub struct CardComposerArgs {
    /// Edit the card in the composer before writing it.
    ///
    /// Bails when neither `card.composer` nor `--composer` names one.
    #[arg(short, long)]
    pub interactive: bool,
    /// Command the card is edited in, overriding `card.composer`.
    ///
    /// A shell line, spawned on the path of a temporary vCard file it
    /// edits in place.
    #[arg(long, value_name = "COMMAND", requires = "interactive")]
    pub composer: Option<String>,
}
