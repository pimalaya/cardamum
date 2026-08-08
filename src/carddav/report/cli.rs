use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::carddav::{
    client::CarddavClient,
    report::{
        multiget::CarddavReportMultigetCommand, query::CarddavReportQueryCommand,
        raw::CarddavReportRawCommand, sync::CarddavReportSyncCommand,
    },
};

/// Run a CardDAV REPORT (RFC 6352 §8, RFC 6578).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum CarddavReportCommand {
    Query(CarddavReportQueryCommand),
    Multiget(CarddavReportMultigetCommand),
    Sync(CarddavReportSyncCommand),
    Raw(CarddavReportRawCommand),
}

impl CarddavReportCommand {
    pub fn execute(self, printer: &mut impl Printer, client: CarddavClient) -> Result<()> {
        match self {
            Self::Query(cmd) => cmd.execute(printer, client),
            Self::Multiget(cmd) => cmd.execute(printer, client),
            Self::Sync(cmd) => cmd.execute(printer, client),
            Self::Raw(cmd) => cmd.execute(printer, client),
        }
    }
}
