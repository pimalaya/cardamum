use anyhow::{Result, bail};
use clap::Parser;
use io_jmap::rfc9610::address_book::set::JmapAddressBookSetArgs;
use pimalaya_cli::printer::{Message, Printer};

use crate::jmap::client::JmapClient;

/// Destroy a JMAP AddressBook (`AddressBook/set` destroy).
///
/// JSON output: `{"message": "..."}`.
#[derive(Debug, Parser)]
pub struct JmapAddressBookDestroyCommand {
    /// AddressBook id.
    #[arg(value_name = "ID")]
    pub id: String,
    /// Also destroy the ContactCards it contains (RFC 9610 §2.3).
    #[arg(long)]
    pub remove_contents: bool,
}

impl JmapAddressBookDestroyCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let args = JmapAddressBookSetArgs {
            destroy: Some(vec![self.id.clone()]),
            on_destroy_remove_contents: Some(self.remove_contents),
            ..Default::default()
        };
        let out = client.address_book_set(args)?;

        if let Some(err) = out.not_destroyed.into_values().next() {
            bail!("AddressBook destroy rejected: {err:?}");
        }

        printer.out(Message::new(format!(
            "AddressBook `{}` successfully destroyed",
            self.id
        )))
    }
}
