//! # Addressbook delete command
//!
//! Deletes an addressbook and every card it contains.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::capability::Capability;
use crate::shared::{
    client::AddressbookClient,
    receipt::{Write, WriteReceipt},
};

/// Delete the given addressbook and every card it contains.
///
/// The addressbook is named explicitly, with no default, deletion being
/// destructive.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct AddressbookDeleteCommand {
    /// Addressbook to delete, by id.
    #[arg(value_name = "ADDRESSBOOK", required = true)]
    pub addressbook: Option<String>,
    /// The addressbook as an option, the v0.3 spelling of the positional.
    #[arg(
        short = 'k',
        long = "addressbook",
        value_name = "ADDRESSBOOK",
        hide = true,
        conflicts_with = "addressbook"
    )]
    pub addressbook_flag: Option<String>,
}

impl AddressbookDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut AddressbookClient) -> Result<()> {
        // NOTE: clap waives the required positional when the conflicting
        // option is present, so exactly one of the two holds the id.
        let id = self
            .addressbook
            .or(self.addressbook_flag)
            .unwrap_or_default();

        client.require(&[Capability::AddressbookDelete])?;
        client.delete_addressbook(&id)?;

        printer.out(WriteReceipt {
            id,
            queued: client.queues_writes(),
            what: "Addressbook",
            write: Write::Delete,
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::AddressbookDeleteCommand;

    fn parse(args: &[&str]) -> Result<Option<String>, ErrorKind> {
        AddressbookDeleteCommand::try_parse_from([&["delete"], args].concat())
            .map(|cmd| cmd.addressbook.or(cmd.addressbook_flag))
            .map_err(|err| err.kind())
    }

    /// The addressbook is a positional, the v0.3 `-k` a hidden spelling
    /// of it, and the two together a conflict.
    #[test]
    fn the_addressbook_is_a_positional_and_k_its_hidden_spelling() {
        assert_eq!(parse(&["work"]), Ok(Some(String::from("work"))));
        assert_eq!(parse(&["-k", "work"]), Ok(Some(String::from("work"))));
        assert_eq!(
            parse(&["--addressbook", "work"]),
            Ok(Some(String::from("work")))
        );
        assert_eq!(
            parse(&["work", "-k", "home"]),
            Err(ErrorKind::ArgumentConflict)
        );
        assert_eq!(parse(&[]), Err(ErrorKind::MissingRequiredArgument));

        let usage = AddressbookDeleteCommand::command()
            .render_usage()
            .to_string();
        assert_eq!(usage, "Usage: cardamum <ADDRESSBOOK>");
        let help = AddressbookDeleteCommand::command()
            .render_long_help()
            .to_string();
        assert!(!help.contains("-k"), "{help}");
    }
}
