//! # REPORT command
//!
//! Dispatches the typed CardDAV REPORTs and the raw escape hatch.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::carddav::{
    client::{CarddavClient, check_addressbook_id, check_card_id},
    report::{
        multiget::CarddavReportMultigetCommand, query::CarddavReportQueryCommand,
        raw::CarddavReportRawCommand, sync::CarddavReportSyncCommand,
    },
};

/// Run a CardDAV REPORT (RFC 6352 §8, RFC 6578).
///
/// The typed subcommands cover the three standard reports; `raw` sends
/// an arbitrary XML body for anything else.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum CarddavReportCommand {
    Query(CarddavReportQueryCommand),
    Multiget(CarddavReportMultigetCommand),
    Sync(CarddavReportSyncCommand),
    Raw(CarddavReportRawCommand),
}

impl CarddavReportCommand {
    /// Refuses, before anything connects, an address book or card id no
    /// CardDAV resource can have (see [`CarddavCommand::check`]).
    ///
    /// [`CarddavCommand::check`]: crate::carddav::cli::CarddavCommand::check
    pub fn check(&self) -> Result<()> {
        match self {
            Self::Query(cmd) => check_addressbook_id(&cmd.addressbook_id),
            Self::Multiget(cmd) => {
                check_addressbook_id(&cmd.addressbook_id)?;
                cmd.card_ids.iter().try_for_each(|id| check_card_id(id))
            }
            Self::Sync(cmd) => check_addressbook_id(&cmd.addressbook_id),
            Self::Raw(cmd) => check_addressbook_id(&cmd.addressbook_id),
        }
    }

    pub fn execute(self, printer: &mut impl Printer, client: CarddavClient) -> Result<()> {
        match self {
            Self::Query(cmd) => cmd.execute(printer, client),
            Self::Multiget(cmd) => cmd.execute(printer, client),
            Self::Sync(cmd) => cmd.execute(printer, client),
            Self::Raw(cmd) => cmd.execute(printer, client),
        }
    }
}
