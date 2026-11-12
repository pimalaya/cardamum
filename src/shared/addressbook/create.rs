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
/// JSON output: `{"id"}`, the identifier the backend assigned.
#[derive(Debug, Parser)]
pub struct AddressbookCreateCommand {
    /// Display name of the addressbook to create.
    #[arg(value_name = "NAME")]
    pub name: String,
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
        if self.description.is_some() {
            capabilities.push(Capability::AddressbookDescription);
        }
        if self.color.is_some() {
            capabilities.push(Capability::AddressbookColor);
        }
        client.require(&capabilities)?;

        let id = client.create_addressbook(
            &self.name,
            self.description.as_deref(),
            self.color.as_deref(),
        )?;

        printer.out(AddressbookCreateOutput { id })
    }
}

/// The addressbook the backend created.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddressbookCreateOutput {
    /// Backend-assigned identifier of the new addressbook.
    pub id: String,
}

impl fmt::Display for AddressbookCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Addressbook `{}` successfully created",
            sanitize::text(&self.id)
        )
    }
}
