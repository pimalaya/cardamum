//! # Card list command
//!
//! Lists a page of cards as a colored table or as JSON, previewing the
//! main vCard properties of each.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, Row, Table},
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    arg::AddressbookIdArg, card::Card, client::AddressbookClient, table::style_from_preset,
};

/// List vCards inside the given addressbook.
///
/// JSON output: `{"cards": [{"id", "addressbookId", "etag", "fnValue",
/// "email", "tel"}]}`.
#[derive(Debug, Parser)]
pub struct CardListCommand {
    /// Addressbook to list the cards of.
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// 1-indexed page number to fetch.
    #[arg(short, long, value_name = "N", default_value_t = 1)]
    pub page: u32,
    /// Maximum number of cards per page.
    ///
    /// Falls back to the `card.list.page-size` config when omitted, then
    /// to 25.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
}

impl CardListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: AddressbookClient) -> Result<()> {
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let page_size = self
            .page_size
            .unwrap_or(client.account.cards_list_page_size());
        let cards = client.list_cards(&addressbook_id, Some(self.page), Some(page_size))?;

        let table = CardListOutput {
            preset: client.account.table_preset().to_string(),
            id_color: client.account.cards_list_table_id_color(),
            fn_color: client.account.cards_list_table_fn_color(),
            email_color: client.account.cards_list_table_email_color(),
            tel_color: client.account.cards_list_table_tel_color(),
            rows: cards.into_iter().map(CardRow::from).collect(),
        };

        printer.out(table)
    }
}

/// Table of cards, and the JSON shape the command prints.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardListOutput {
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// Color of the ID column.
    #[serde(skip)]
    pub id_color: Color,
    /// Color of the FN column.
    #[serde(skip)]
    pub fn_color: Color,
    /// Color of the EMAIL column.
    #[serde(skip)]
    pub email_color: Color,
    /// Color of the TEL column.
    #[serde(skip)]
    pub tel_color: Color,
    /// The cards of the requested page.
    #[serde(rename = "cards")]
    pub rows: Vec<CardRow>,
}

/// One card, previewed as rendered by [`CardListOutput`].
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardRow {
    /// Backend-specific identifier of the card.
    pub id: String,
    /// Identifier of the addressbook holding it.
    pub addressbook_id: String,
    /// Entity tag, when the backend exposes one.
    pub etag: Option<String>,
    /// First `FN` value found in the vCard.
    pub fn_value: Option<String>,
    /// First `EMAIL` value found in the vCard.
    pub email: Option<String>,
    /// First `TEL` value found in the vCard.
    pub tel: Option<String>,
}

impl From<Card> for CardRow {
    fn from(card: Card) -> Self {
        let (fn_value, email, tel) = vcard_preview(&card.contents);
        Self {
            id: card.id,
            addressbook_id: card.addressbook_id,
            etag: card.etag,
            fn_value,
            email,
            tel,
        }
    }
}

impl fmt::Display for CardListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("FN"),
                Cell::new("EMAIL"),
                Cell::new("TEL"),
            ]))
            .add_rows(self.rows.iter().map(|card| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&card.id).fg(self.id_color))
                    .add_cell(Cell::new(card.fn_value.as_deref().unwrap_or("")).fg(self.fn_color))
                    .add_cell(Cell::new(card.email.as_deref().unwrap_or("")).fg(self.email_color))
                    .add_cell(Cell::new(card.tel.as_deref().unwrap_or("")).fg(self.tel_color));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}

/// Pulls the first `FN`, `EMAIL` and `TEL` value out of the raw bytes.
///
/// A scan rather than a parse: the listing renders three columns, which
/// does not justify parsing every vCard in the page.
fn vcard_preview(bytes: &[u8]) -> (Option<String>, Option<String>, Option<String>) {
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return (None, None, None),
    };

    let mut fn_value = None;
    let mut email = None;
    let mut tel = None;

    // NOTE: the three reads are independent. Chained, a line that failed
    // the EMAIL read never reached the TEL one, so a card writing its TEL
    // above its EMAIL listed with an empty TEL column.
    for line in text.lines() {
        let line = line.trim();

        if fn_value.is_none() && line.starts_with("FN:") {
            fn_value = Some(line[3..].to_string());
        }
        if email.is_none()
            && let Some(rest) = pick_property(line, "EMAIL")
        {
            email = Some(rest);
        }
        if tel.is_none()
            && let Some(rest) = pick_property(line, "TEL")
        {
            tel = Some(rest);
        }
    }

    (fn_value, email, tel)
}

/// Returns the value of a property line, skipping over its parameters.
fn pick_property(line: &str, key: &str) -> Option<String> {
    if !line.starts_with(key) {
        return None;
    }

    let after_key = &line[key.len()..];
    let colon = after_key.find(':')?;
    Some(after_key[colon + 1..].to_string())
}
