use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::carddav::{
    client::CarddavClient, delete::CarddavDeleteCommand, discover::CarddavDiscoverCommand,
    get::CarddavGetCommand, mkcol::CarddavMkcolCommand, propfind::CarddavPropfindCommand,
    proppatch::CarddavProppatchCommand, put::CarddavPutCommand, report::cli::CarddavReportCommand,
};

/// CardDAV-specific API.
///
/// A flat list of WebDAV / CardDAV methods (RFC 4918 / 5689 / 6352 /
/// 6578) run directly against the active account's endpoint, plus the
/// discovery chain. Each command is named after its protocol
/// counterpart and surfaces the native ids, ETags, CTags and
/// sync-tokens the shared `addressbook` / `card` API hides.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum CarddavCommand {
    Discover(CarddavDiscoverCommand),
    Propfind(CarddavPropfindCommand),
    Proppatch(CarddavProppatchCommand),
    Mkcol(CarddavMkcolCommand),
    #[command(subcommand)]
    Report(CarddavReportCommand),
    Get(CarddavGetCommand),
    Put(CarddavPutCommand),
    Delete(CarddavDeleteCommand),
}

impl CarddavCommand {
    pub fn execute(self, printer: &mut impl Printer, client: CarddavClient) -> Result<()> {
        match self {
            Self::Discover(cmd) => cmd.execute(printer, client),
            Self::Propfind(cmd) => cmd.execute(printer, client),
            Self::Proppatch(cmd) => cmd.execute(printer, client),
            Self::Mkcol(cmd) => cmd.execute(printer, client),
            Self::Report(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Put(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        }
    }
}
