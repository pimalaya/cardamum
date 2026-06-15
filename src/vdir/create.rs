use anyhow::Result;
use clap::Parser;
use io_vdir::collection::Collection;
use pimalaya_cli::printer::{Message, Printer};

use crate::vdir::client::VdirClient;

/// Create a new vdir collection under the configured root.
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct VdirCollectionCreateCommand {
    /// Name of the collection. Used both as on-disk directory name and
    /// as the `displayname` metadata file content.
    #[arg(value_name = "NAME")]
    pub name: String,
    /// Optional free-form description (`description` metadata file).
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
    /// Optional ASCII `#RRGGBB` color (`color` metadata file).
    #[arg(short, long, value_name = "HEX")]
    pub color: Option<String>,
}

impl VdirCollectionCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.root().join(&self.name);
        let collection = Collection {
            path,
            display_name: Some(self.name.clone()),
            description: self.description,
            color: self.color,
        };

        client.create_collection(collection)?;

        printer.out(Message::new(format!(
            "Collection `{}` successfully created",
            self.name
        )))
    }
}
