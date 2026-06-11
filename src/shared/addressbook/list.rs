use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, Row, Table};
use io_addressbook::addressbook::Addressbook;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::shared::client::AddressbookClient;

/// List every addressbook available to the active account.
#[derive(Debug, Parser)]
pub struct AddressbookListCommand;

impl AddressbookListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbooks = client.list_addressbooks()?;

        let table = AddressbooksTable {
            preset: client.account.table_preset().to_string(),
            id_color: client.account.addressbooks_list_table_id_color(),
            name_color: client.account.addressbooks_list_table_name_color(),
            description_color: client.account.addressbooks_list_table_description_color(),
            color_color: client.account.addressbooks_list_table_color_color(),
            rows: addressbooks.into_iter().map(From::from).collect(),
        };

        printer.out(table)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AddressbooksTable {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(skip)]
    pub name_color: Color,
    #[serde(skip)]
    pub description_color: Color,
    #[serde(skip)]
    pub color_color: Color,
    #[serde(rename = "addressbooks")]
    pub rows: Vec<AddressbookRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AddressbookRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

impl From<Addressbook> for AddressbookRow {
    fn from(book: Addressbook) -> Self {
        Self {
            id: book.id,
            name: book.name,
            description: book.description,
            color: book.color,
        }
    }
}

impl fmt::Display for AddressbooksTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("DESC"),
                Cell::new("COLOR"),
            ]))
            .add_rows(self.rows.iter().map(|book| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&book.id).fg(self.id_color))
                    .add_cell(Cell::new(&book.name).fg(self.name_color))
                    .add_cell(
                        Cell::new(book.description.as_deref().unwrap_or(""))
                            .fg(self.description_color),
                    )
                    .add_cell(Cell::new(book.color.as_deref().unwrap_or("")).fg(self.color_color));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}
