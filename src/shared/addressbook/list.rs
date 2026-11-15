//! # Addressbook list command
//!
//! Lists the addressbooks the account exposes, as aligned columns or as
//! JSON.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{listing::Listing, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::backend::Backend;
use crate::capability::Capability;
use crate::error::{ItemError, partial_when};
use crate::shared::{addressbook::Addressbook, client::AddressbookClient};

/// List every addressbook available to the active account.
///
/// Each line shows the id of an address book, `default` on the one the
/// server names the default, its color, then its name, under a header
/// naming the account. An address book the backend could not read is
/// named by an `Error:` line at the end, and the command exits 3.
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
            account: client.name.clone(),
            backend: client.backend(),
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
    /// The name of the account listed.
    #[serde(skip)]
    pub account: String,
    /// The backend serving the account.
    #[serde(skip)]
    pub backend: Backend,
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
        let backend = self.backend.to_string();
        let mut listing = Listing::new(&self.account, Some(&backend));

        // NOTE: a name is free text, so it reads last; a description is
        // left to the JSON.
        for book in &self.rows {
            let default = match book.default {
                true => String::from("default"),
                false => String::new(),
            };

            listing.items.push([
                sanitize::text(&book.id).into_owned(),
                default,
                sanitize::text(book.color.as_deref().unwrap_or_default()).into_owned(),
                sanitize::text(&book.name).into_owned(),
            ]);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
    }
}

#[cfg(all(test, feature = "vdir"))]
mod tests {
    use anyhow::anyhow;

    use super::*;

    fn output() -> AddressbookListOutput {
        AddressbookListOutput {
            account: String::from("perso"),
            backend: Backend::Vdir,
            rows: vec![
                AddressbookRow {
                    id: String::from("personal"),
                    name: String::from("Personal\x1b[2J"),
                    description: Some(String::from("Family")),
                    color: Some(String::from("#ff0000")),
                    default: true,
                },
                AddressbookRow {
                    id: String::from("work"),
                    name: String::from("Work"),
                    description: None,
                    color: None,
                    default: false,
                },
            ],
            errors: Vec::new(),
        }
    }

    /// The address books print one line each under the account, as
    /// calendula's calendars and Himalaya's mailboxes do: no header row,
    /// the id first, the name last, `-` for an empty cell.
    #[test]
    fn address_books_print_one_line_each_under_the_account() {
        let expected = "\
Account:    perso (vdir)

personal  default  #ff0000  Personal\u{fffd}[2J
work      -        -        Work
";
        assert_eq!(output().to_string(), expected);
    }

    /// The errors print last, after the lines and one blank line.
    #[test]
    fn the_errors_print_last_after_one_blank_line() {
        let mut output = output();
        output.rows.truncate(1);
        output.errors = vec![
            ItemError::new(Some("work"), &anyhow!("Cannot read it")),
            ItemError::new(None, &anyhow!("Gone")),
        ];

        let expected = "\
Account:    perso (vdir)

personal  default  #ff0000  Personal\u{fffd}[2J

Error:  work  -  Cannot read it
Error:  -     -  Gone
";
        assert_eq!(output.to_string(), expected);

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["errors"][0]["id"], "work");
        assert_eq!(json["errors"][1]["id"], serde_json::Value::Null);
    }
}
