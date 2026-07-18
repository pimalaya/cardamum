use std::collections::BTreeMap;

use anyhow::{Result, bail};
use clap::Parser;
use io_jmap::rfc9610::address_book::set::{JmapAddressBookCreate, JmapAddressBookSetArgs};
use pimalaya_cli::printer::Printer;

use crate::jmap::{client::JmapClient, render::BookReport};

/// Create a JMAP AddressBook (`AddressBook/set` create).
///
/// JSON output: the raw JMAP AddressBook the server created.
#[derive(Debug, Parser)]
pub struct JmapAddressBookCreateCommand {
    /// Name of the AddressBook.
    #[arg(value_name = "NAME")]
    pub name: String,
    /// Optional long-form description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
}

impl JmapAddressBookCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let create = BTreeMap::from([(
            "c0".to_string(),
            JmapAddressBookCreate {
                name: Some(self.name),
                description: self.description,
                ..Default::default()
            },
        )]);
        let args = JmapAddressBookSetArgs {
            create: Some(create),
            ..Default::default()
        };
        let out = client.address_book_set(args)?;

        if let Some(err) = out.not_created.into_values().next() {
            bail!("AddressBook create rejected: {err:?}");
        }
        let created = out
            .created
            .into_values()
            .next()
            .ok_or_else(|| anyhow::anyhow!("AddressBook create returned no object"))?;

        printer.out(BookReport(created))
    }
}
