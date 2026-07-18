use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::vdir::{client::VdirClient, item::input::kind_str};

/// Read the raw bytes of the given item.
///
/// JSON output: `{"id", "kind", "contents"}`, with the raw item in
/// `contents`.
#[derive(Debug, Parser)]
pub struct VdirItemGetCommand {
    /// Collection the item lives in.
    #[arg(value_name = "COLLECTION")]
    pub collection: String,
    /// Item id (file stem, without extension).
    #[arg(value_name = "ID")]
    pub id: String,
}

impl VdirItemGetCommand {
    pub fn execute(self, printer: &mut impl Printer, client: VdirClient) -> Result<()> {
        let path = client.collection_path(&self.collection)?;
        let item = client.get_item(path, &self.id)?;

        let out = Item {
            id: self.id,
            kind: kind_str(item.kind),
            contents: String::from_utf8(item.contents)?,
        };

        printer.out(out)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub id: String,
    pub kind: &'static str,
    pub contents: String,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents)
    }
}
