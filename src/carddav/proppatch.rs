//! # PROPPATCH command
//!
//! Sets the DAV properties of an addressbook collection with a
//! `PROPPATCH`.

use anyhow::{Result, bail};
use clap::Parser;
use io_webdav::rfc6352::addressbook::CarddavAddressbookPatch;
use pimalaya_cli::printer::{Message, Printer};

use crate::carddav::client::CarddavClient;

/// PROPPATCH: set properties on an addressbook collection.
///
/// Only the flags you pass are sent (a PROPPATCH `set`), so unset
/// properties are left untouched; clearing a property is not exposed.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct CarddavProppatchCommand {
    /// Identifier (last URL segment) of the addressbook to patch.
    #[arg(value_name = "ID")]
    pub id: String,
    /// New human-readable name (DAV:displayname).
    #[arg(short, long, value_name = "TEXT")]
    pub name: Option<String>,
    /// New free-form description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
    /// New ASCII `#RRGGBB` color marker.
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,
}

impl CarddavProppatchCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        if self.name.is_none() && self.description.is_none() && self.color.is_none() {
            bail!("Nothing to patch: pass at least one of --name, --description or --color");
        }

        let patch = CarddavAddressbookPatch {
            id: self.id.clone(),
            display_name: self.name.map(Some),
            description: self.description.map(Some),
            color: self.color.map(Some),
        };

        client.update_addressbook(&patch)?;

        printer.out(Message::new(format!(
            "Addressbook `{}` properties successfully patched",
            self.id
        )))
    }
}
