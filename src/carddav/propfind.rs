use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::{Message, Printer};

use crate::carddav::client::CarddavClient;

/// Read a single card via PROPFIND-style retrieval.
///
/// Helper around RFC 6352 `GET`: reads the raw vCard bytes and prints
/// the ETag, leaving the body on stdout when `--raw` is set.
#[derive(Debug, Parser)]
pub struct CarddavPropfindCommand {
    /// Identifier of the parent addressbook.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Card UID.
    #[arg(value_name = "ID")]
    pub card_id: String,
    /// Print the raw vCard body instead of the ETag.
    #[arg(long)]
    pub raw: bool,
}

impl CarddavPropfindCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let body = client.read_card(&self.addressbook_id, &self.card_id)?;

        if self.raw {
            printer.out(Message::new(
                String::from_utf8_lossy(&body.data).into_owned(),
            ))
        } else {
            printer.out(Message::new(
                body.etag.unwrap_or_else(|| "(no etag)".into()),
            ))
        }
    }
}
