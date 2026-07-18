use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::jmap::{
    address_book::cli::JmapAddressBookCommand, client::JmapClient,
    contact_card::cli::JmapContactCardCommand, session::cli::JmapSessionCommand,
};

/// JMAP-specific API (RFC 8620 + RFC 9610).
///
/// Nested by JMAP object type, each command named after its JMAP method
/// (`/get`, `/query`, `/set` create/update/destroy, `/changes`,
/// `/copy`). Works with the raw JSContact model: `create` / `update`
/// take a JSContact JSON body and `--json` prints the raw JMAP payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum JmapCommand {
    #[command(subcommand, visible_aliases = ["abook", "addressbook"])]
    AddressBook(JmapAddressBookCommand),
    #[command(subcommand, visible_aliases = ["card", "cards"])]
    ContactCard(JmapContactCardCommand),
    #[command(subcommand)]
    Session(JmapSessionCommand),
}

impl JmapCommand {
    pub fn execute(self, printer: &mut impl Printer, client: JmapClient) -> Result<()> {
        match self {
            Self::AddressBook(cmd) => cmd.execute(printer, client),
            Self::ContactCard(cmd) => cmd.execute(printer, client),
            Self::Session(cmd) => cmd.execute(printer, client),
        }
    }
}
