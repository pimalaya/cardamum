use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::carddav::{
    client::CarddavClient, create::CarddavAddressbookCreateCommand,
    delete::CarddavAddressbookDeleteCommand, discover::CarddavDiscoverCommand,
    list::CarddavAddressbookListCommand, propfind::CarddavPropfindCommand,
    report::CarddavReportCommand,
};

/// CardDAV-specific API.
///
/// Exposes RFC 4918 / 6352 verbs directly against the active account's CardDAV
/// endpoint, plus the discovery chain (well-known + current-user-principal +
/// addressbook-home-set).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum CarddavCommand {
    Discover(CarddavDiscoverCommand),
    Propfind(CarddavPropfindCommand),
    Report(CarddavReportCommand),
    List(CarddavAddressbookListCommand),
    Create(CarddavAddressbookCreateCommand),
    Delete(CarddavAddressbookDeleteCommand),
}

impl CarddavCommand {
    pub fn execute(self, printer: &mut impl Printer, client: CarddavClient) -> Result<()> {
        match self {
            Self::Discover(cmd) => cmd.execute(printer, client),
            Self::Propfind(cmd) => cmd.execute(printer, client),
            Self::Report(cmd) => cmd.execute(printer, client),
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
