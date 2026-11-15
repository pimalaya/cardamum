//! # Addressbook create command
//!
//! Creates an addressbook on the account backend and prints its new
//! identifier.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::Capability;
use crate::shared::client::AddressbookClient;

/// Create a new addressbook.
///
/// The reported id is the one later commands address the addressbook by.
/// Given only a name, vdir names its directory and CardDAV its URL segment
/// after it; `--id` chooses that id instead, the name staying the display
/// name. JMAP, Graph and People mint their own ids, so they refuse `--id`.
///
/// JSON output: `{"id", "name", "queued"}`, the identifier the backend
/// assigned, the name given and whether a sync engine creates it later.
#[derive(Debug, Parser)]
pub struct AddressbookCreateCommand {
    /// Display name of the addressbook to create.
    #[arg(value_name = "NAME")]
    pub name: String,
    /// Identifier of the addressbook, on the backends letting a client
    /// choose one: its vdir directory or its CardDAV URL segment.
    ///
    /// Omitted, the name is the id, and must then be one.
    #[arg(long, value_name = "ID")]
    pub id: Option<String>,
    /// Free-form description of the addressbook.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,
    /// ASCII `#RRGGBB` color marker of the addressbook.
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,
}

impl AddressbookCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut AddressbookClient) -> Result<()> {
        let mut capabilities = vec![Capability::AddressbookCreate];
        if self.id.is_some() {
            capabilities.push(Capability::AddressbookCreateId);
        }
        if self.description.is_some() {
            capabilities.push(Capability::AddressbookDescription);
        }
        if self.color.is_some() {
            capabilities.push(Capability::AddressbookColor);
        }
        client.require(&capabilities)?;

        let id = client.create_addressbook(
            self.id.as_deref(),
            &self.name,
            self.description.as_deref(),
            self.color.as_deref(),
        )?;

        printer.out(AddressbookCreateOutput {
            id,
            name: self.name,
            queued: client.queues_writes(),
        })
    }
}

/// The addressbook the backend created.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddressbookCreateOutput {
    /// Backend-assigned identifier of the new addressbook.
    pub id: String,
    /// The display name it was created with.
    pub name: String,
    /// Whether a sync engine applies the write later rather than now.
    pub queued: bool,
}

impl fmt::Display for AddressbookCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = sanitize::text(&self.id);
        match self.queued {
            true => writeln!(
                f,
                "Addressbook `{id}` queued for creation; the next sync creates it"
            ),
            false => writeln!(f, "Addressbook `{id}` successfully created"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AddressbookCreateOutput;

    /// The receipt names the address book created, by its id and the name
    /// it was given, and says whether a sync creates it.
    #[test]
    fn the_receipt_carries_the_name_and_queued() {
        let receipt = AddressbookCreateOutput {
            id: String::from("AAMk"),
            name: String::from("Work"),
            queued: false,
        };

        let text = "Addressbook `AAMk` successfully created\n";
        assert_eq!(receipt.to_string(), text);
        assert_eq!(
            serde_json::to_string(&receipt).unwrap(),
            r#"{"id":"AAMk","name":"Work","queued":false}"#
        );
    }
}
