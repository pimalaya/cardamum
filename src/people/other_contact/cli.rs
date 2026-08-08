use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::people::{
    client::PeopleClient,
    other_contact::{
        copy::PeopleOtherContactCopyCommand, list::PeopleOtherContactListCommand,
        search::PeopleOtherContactSearchCommand,
    },
};

/// The "other contacts": people the user has interacted with but not
/// added to their contacts (`otherContacts`). Read-only, except `copy`.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum PeopleOtherContactCommand {
    List(PeopleOtherContactListCommand),
    Search(PeopleOtherContactSearchCommand),
    Copy(PeopleOtherContactCopyCommand),
}

impl PeopleOtherContactCommand {
    pub fn execute(self, printer: &mut impl Printer, client: PeopleClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Search(cmd) => cmd.execute(printer, client),
            Self::Copy(cmd) => cmd.execute(printer, client),
        }
    }
}
