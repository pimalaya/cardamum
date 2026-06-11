use anyhow::Result;
use clap::Parser;
use io_webdav::rfc6352::addressbook::Addressbook as WireAddressbook;
use pimalaya_cli::printer::{Message, Printer};

use crate::carddav::client::CarddavClient;

/// Create an addressbook collection on the server.
#[derive(Debug, Parser)]
pub struct CarddavAddressbookCreateCommand {
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
    #[arg(short, long, value_name = "HEX")]
    pub color: Option<String>,
}

impl CarddavAddressbookCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let wire = WireAddressbook {
            id: self.id.clone(),
            display_name: self.name,
            description: self.description,
            color: self.color,
        };

        client.create_addressbook(&wire)?;

        printer.out(Message::new(format!(
            "Addressbook `{}` successfully created",
            self.id
        )))
    }
}
