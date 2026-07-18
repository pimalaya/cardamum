use anyhow::Result;
use clap::Parser;
use io_webdav::rfc6352::addressbook::Addressbook;
use pimalaya_cli::printer::{Message, Printer};

use crate::carddav::client::CarddavClient;

/// MKCOL: create an addressbook collection on the server (RFC 5689
/// extended MKCOL).
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
#[command(visible_alias = "create")]
pub struct CarddavMkcolCommand {
    /// Identifier (last URL segment) of the addressbook to create.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Optional human-readable name (DAV:displayname).
    #[arg(short, long, value_name = "TEXT")]
    pub name: Option<String>,
    /// Optional free-form description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
    /// Optional ASCII `#RRGGBB` color marker.
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,
}

impl CarddavMkcolCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let wire = Addressbook {
            id: self.id.clone(),
            display_name: self.name,
            description: self.description,
            color: self.color,
            ctag: None,
            sync_token: None,
        };

        client.create_addressbook(&wire)?;

        printer.out(Message::new(format!(
            "Addressbook `{}` successfully created",
            self.id
        )))
    }
}
