use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::google::{
    client::GoogleClient,
    other_contact::{
        copy::GoogleOtherContactCopyCommand, list::GoogleOtherContactListCommand,
        search::GoogleOtherContactSearchCommand,
    },
};

/// The "other contacts": people the user has interacted with but not
/// added to their contacts (`otherContacts`). Read-only, except `copy`.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GoogleOtherContactCommand {
    List(GoogleOtherContactListCommand),
    Search(GoogleOtherContactSearchCommand),
    Copy(GoogleOtherContactCopyCommand),
}

impl GoogleOtherContactCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GoogleClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Search(cmd) => cmd.execute(printer, client),
            Self::Copy(cmd) => cmd.execute(printer, client),
        }
    }
}
