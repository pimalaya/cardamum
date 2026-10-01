//! # Contact group commands
//!
//! Dispatches the contact group commands to the People operation each
//! runs.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::gpeople::{
    client::GpeopleClient,
    contact_group::{
        create::GpeopleContactGroupCreateCommand, delete::GpeopleContactGroupDeleteCommand,
        get::GpeopleContactGroupGetCommand, list::GpeopleContactGroupListCommand,
        members::GpeopleContactGroupMembersCommand, update::GpeopleContactGroupUpdateCommand,
    },
};

/// Manage People contact groups (the addressbooks).
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum GpeopleContactGroupCommand {
    List(GpeopleContactGroupListCommand),
    Get(GpeopleContactGroupGetCommand),
    #[command(visible_aliases = ["add", "new"])]
    Create(GpeopleContactGroupCreateCommand),
    Update(GpeopleContactGroupUpdateCommand),
    #[command(visible_aliases = ["del", "rm"])]
    Delete(GpeopleContactGroupDeleteCommand),
    #[command(alias = "member")]
    Members(GpeopleContactGroupMembersCommand),
}

impl GpeopleContactGroupCommand {
    pub fn execute(self, printer: &mut impl Printer, client: GpeopleClient) -> Result<()> {
        match self {
            Self::List(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Create(cmd) => cmd.execute(printer, client),
            Self::Update(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
            Self::Members(cmd) => cmd.execute(printer, client),
        }
    }
}
