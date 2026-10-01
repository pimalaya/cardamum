//! # Profile commands
//!
//! Dispatches the commands reading the signed-in user.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{client::GpeopleClient, profile::get::GpeopleProfileGetCommand};

/// The signed-in user (`people/me`).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GpeopleProfileCommand {
    Get(GpeopleProfileGetCommand),
}

impl GpeopleProfileCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GpeopleClient) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, client),
        }
    }
}
