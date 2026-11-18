//! # RMCOL command
//!
//! Runs a WebDAV `DELETE` against a whole addressbook collection, the
//! counterpart of `carddav mkcol`.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{carddav::client::CarddavClient, shared::receipt::IdReceipt};

/// DELETE an addressbook collection and every card it contains.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct CarddavRmcolCommand {
    /// Identifier of the addressbook (its last URL segment).
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
}

impl CarddavRmcolCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        client.delete_addressbook(&self.addressbook_id)?;

        printer.out(IdReceipt {
            id: self.addressbook_id,
            done: "Addressbook deleted",
        })
    }
}
