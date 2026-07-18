use std::collections::BTreeMap;

use anyhow::{Result, bail};
use clap::Parser;
use io_jmap::rfc9610::address_book::set::{JmapAddressBookSetArgs, JmapAddressBookUpdate};
use pimalaya_cli::printer::{Message, Printer};

use crate::jmap::{client::JmapClient, render::BookReport};

/// Update a JMAP AddressBook (`AddressBook/set` update).
///
/// JSON output: the raw JMAP AddressBook after the update (or a message
/// when the server returns no object).
#[derive(Debug, Parser)]
pub struct JmapAddressBookUpdateCommand {
    /// AddressBook id.
    #[arg(value_name = "ID")]
    pub id: String,
    /// New name.
    #[arg(short, long, value_name = "TEXT")]
    pub name: Option<String>,
    /// New description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
}

impl JmapAddressBookUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let update = BTreeMap::from([(
            self.id.clone(),
            JmapAddressBookUpdate {
                name: self.name,
                description: self.description,
                ..Default::default()
            },
        )]);
        let args = JmapAddressBookSetArgs {
            update: Some(update),
            ..Default::default()
        };
        let out = client.address_book_set(args)?;

        if let Some(err) = out.not_updated.into_values().next() {
            bail!("AddressBook update rejected: {err:?}");
        }

        match out.updated.into_values().next().flatten() {
            Some(book) => printer.out(BookReport(book)),
            None => printer.out(Message::new(format!(
                "AddressBook `{}` successfully updated",
                self.id
            ))),
        }
    }
}
