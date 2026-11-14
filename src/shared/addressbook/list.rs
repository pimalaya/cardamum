//! # Addressbook list command
//!
//! Lists the addressbooks the account exposes, as aligned columns or as
//! JSON.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::Capability;
use crate::error::{ItemError, partial_when, write_item_errors};
use crate::shared::{addressbook::Addressbook, client::AddressbookClient};

/// List every addressbook available to the active account.
///
/// JSON output: `{"addressbooks": [{"id", "name", "description",
/// "color", "default"}], "errors": [{"id", "code", "error", "sources"}]}`:
/// the address books, then the error of each one the account could not
/// read, `[]` when none, the command then exiting 3.
#[derive(Debug, Parser)]
pub struct AddressbookListCommand;

impl AddressbookListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut AddressbookClient) -> Result<()> {
        client.require(&[Capability::AddressbookList])?;
        let addressbooks = client.list_addressbooks()?;

        let output = AddressbookListOutput {
            rows: addressbooks.into_iter().map(From::from).collect(),
            errors: Vec::new(),
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// The addressbooks of the account, and the JSON shape the command prints.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddressbookListOutput {
    /// The addressbooks the account exposes.
    #[serde(rename = "addressbooks")]
    pub rows: Vec<AddressbookRow>,
    /// The error of each address book the account could not read, in the
    /// order met.
    pub errors: Vec<ItemError>,
}

/// One addressbook, as rendered by [`AddressbookListOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddressbookRow {
    /// Backend-specific identifier of the collection.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Free-form description, when the backend exposes one.
    pub description: Option<String>,
    /// ASCII `#RRGGBB` color marker, when the backend exposes one.
    pub color: Option<String>,
    /// Whether new cards go here when no address book is named, as the
    /// server states it.
    pub default: bool,
}

impl From<Addressbook> for AddressbookRow {
    fn from(book: Addressbook) -> Self {
        Self {
            id: book.id,
            name: book.name,
            description: book.description,
            color: book.color,
            default: book.default,
        }
    }
}

impl fmt::Display for AddressbookListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: a name is free text, so it reads last; a description is
        // left to the JSON.
        let mut columns = Columns::new(["ID", "DEFAULT", "COLOR", "NAME"]);

        for book in &self.rows {
            let default = if book.default { "yes" } else { "" };
            let color = book.color.as_deref().map(sanitize::text);

            columns.push([
                sanitize::text(&book.id).into_owned(),
                default.to_string(),
                color.map(|color| color.into_owned()).unwrap_or_default(),
                sanitize::text(&book.name).into_owned(),
            ]);
        }

        write!(f, "{columns}")?;
        write_item_errors(f, &self.errors)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;

    fn output() -> AddressbookListOutput {
        AddressbookListOutput {
            rows: vec![AddressbookRow {
                id: String::from("personal"),
                name: String::from("Personal"),
                description: None,
                color: Some(String::from("#ff0000")),
                default: true,
            }],
            errors: Vec::new(),
        }
    }

    /// The table alone when every address book was read.
    #[test]
    fn a_listing_without_errors_prints_its_table_alone() {
        let expected = "\
ID        DEFAULT  COLOR    NAME
personal  yes      #ff0000  Personal
";
        assert_eq!(output().to_string(), expected);
    }

    /// The errors print last, after the table and one blank line.
    #[test]
    fn the_errors_print_last_after_one_blank_line() {
        let mut output = output();
        output.errors = vec![
            ItemError::new(Some("work"), &anyhow!("Cannot read it")),
            ItemError::new(None, &anyhow!("Gone")),
        ];

        let expected = "\
ID        DEFAULT  COLOR    NAME
personal  yes      #ff0000  Personal

Error:  work  -  Cannot read it
Error:  -     -  Gone
";
        assert_eq!(output.to_string(), expected);

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["errors"][0]["id"], "work");
        assert_eq!(json["errors"][1]["id"], serde_json::Value::Null);
    }
}
