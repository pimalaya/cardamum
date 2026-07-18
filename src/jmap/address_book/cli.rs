use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::jmap::{
    address_book::{
        changes::JmapAddressBookChangesCommand, create::JmapAddressBookCreateCommand,
        destroy::JmapAddressBookDestroyCommand, get::JmapAddressBookGetCommand,
        update::JmapAddressBookUpdateCommand,
    },
    client::JmapClient,
};

/// Manage JMAP AddressBooks (RFC 9610 §2).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum JmapAddressBookCommand {
    Get(JmapAddressBookGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(JmapAddressBookCreateCommand),
    Update(JmapAddressBookUpdateCommand),
    #[command(visible_aliases = ["delete", "del", "rm", "remove"])]
    Destroy(JmapAddressBookDestroyCommand),
    Changes(JmapAddressBookChangesCommand),
}

impl JmapAddressBookCommand {
    pub fn execute(self, printer: &mut impl Printer, client: JmapClient) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Destroy(cmd) => cmd.execute(printer, client),
            Self::Changes(cmd) => cmd.execute(printer, client),
        }
    }
}
