//! # Addressbook update command
//!
//! Turns the given flags into a partial update and applies it to an
//! existing addressbook.

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::capability::Capability;
use crate::shared::{
    addressbook::AddressbookDiff,
    client::AddressbookClient,
    receipt::{Write, WriteReceipt},
};

/// Update an existing addressbook.
///
/// Each flag updates its own field and leaves the others untouched. To
/// clear an optional field, pass an empty value, as in `--description ""`.
///
/// JSON output: `{"id", "queued"}`.
#[derive(Debug, Parser)]
pub struct AddressbookUpdateCommand {
    /// Addressbook to update, by id.
    ///
    /// Falls back to the `addressbook.default` config when omitted,
    /// otherwise the command bails.
    #[arg(value_name = "ADDRESSBOOK")]
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
    /// New display name.
    #[arg(long, short, value_name = "NAME")]
    pub name: Option<String>,
    /// New free-form description, `""` to clear it.
    #[arg(long, short, alias = "desc", value_name = "TEXT")]
    pub description: Option<String>,
    /// New ASCII `#RRGGBB` color marker, `""` to clear it.
    #[arg(long, short = 'C', value_name = "HEX")]
    pub color: Option<String>,
}

impl AddressbookUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut AddressbookClient) -> Result<()> {
        let mut capabilities = Vec::new();
        if self.name.is_some() {
            capabilities.push(Capability::AddressbookRename);
        }
        if self.description.is_some() {
            capabilities.push(Capability::AddressbookDescription);
        }
        if self.color.is_some() {
            capabilities.push(Capability::AddressbookColor);
        }
        client.require(&capabilities)?;

        let patch = AddressbookDiff {
            name: self.name,
            description: self.description.map(|d| (!d.is_empty()).then_some(d)),
            color: self.color.map(|c| (!c.is_empty()).then_some(c)),
        };

        if patch == AddressbookDiff::default() {
            bail!("Nothing to update: pass at least one of --name, --description or --color");
        }

        let addressbook = self.addressbook.or(self.addressbook_flag);
        let addressbook_id = client.account.addressbook_id(addressbook)?;
        client.update_addressbook(&addressbook_id, patch)?;

        printer.out(WriteReceipt {
            id: addressbook_id,
            queued: client.queues_writes(),
            what: "Addressbook",
            write: Write::Update,
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::AddressbookUpdateCommand;

    fn parse(args: &[&str]) -> Result<Option<String>, ErrorKind> {
        AddressbookUpdateCommand::try_parse_from([&["update"], args].concat())
            .map(|cmd| cmd.addressbook.or(cmd.addressbook_flag))
            .map_err(|err| err.kind())
    }

    /// The addressbook is an optional positional, the default one taken
    /// without it, the v0.3 `-k` a hidden spelling of it.
    #[test]
    fn the_addressbook_is_a_positional_and_k_its_hidden_spelling() {
        let work = Ok(Some(String::from("work")));
        assert_eq!(parse(&["work", "-n", "Work"]), work);
        assert_eq!(parse(&["-n", "Work", "work"]), work);
        assert_eq!(parse(&["-k", "work", "-n", "Work"]), work);
        assert_eq!(parse(&["-n", "Work"]), Ok(None));
        let conflict = parse(&["work", "-k", "home", "-n", "Work"]);
        assert_eq!(conflict, Err(ErrorKind::ArgumentConflict));

        let usage = AddressbookUpdateCommand::command()
            .render_usage()
            .to_string();
        assert_eq!(usage, "Usage: cardamum [OPTIONS] [ADDRESSBOOK]");
        let help = AddressbookUpdateCommand::command()
            .render_long_help()
            .to_string();
        assert!(!help.contains("-k"), "{help}");
    }

    /// The new display name is a `NAME`, as calendula's `calendar update`
    /// names it.
    #[test]
    fn the_new_name_is_a_name() {
        let help = AddressbookUpdateCommand::command()
            .render_long_help()
            .to_string();
        assert!(help.contains("-n, --name <NAME>"), "{help}");
    }
}
