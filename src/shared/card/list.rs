use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Color, Row, Table};
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::shared::{arg::AddressbookIdArg, card::Card, client::AddressbookClient};

/// List vCards inside the given addressbook.
///
/// JSON output: `{"cards": [{"id", "addressbook_id", "etag", "fn_value",
/// "email", "tel"}]}`.
#[derive(Debug, Parser)]
pub struct CardListCommand {
    #[command(flatten)]
    pub addressbook: AddressbookIdArg,
    /// 1-indexed page number to fetch.
    #[arg(short, long, value_name = "N", default_value_t = 1)]
    pub page: u32,
    /// Maximum number of cards returned per page.
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

        let table = CardsTable {
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

#[derive(Clone, Debug, Serialize)]
pub struct CardsTable {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(skip)]
    pub fn_color: Color,
    #[serde(skip)]
    pub email_color: Color,
    #[serde(skip)]
    pub tel_color: Color,
    #[serde(rename = "cards")]
    pub rows: Vec<CardRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CardRow {
    pub id: String,
    pub addressbook_id: String,
    pub etag: Option<String>,
    pub fn_value: Option<String>,
    pub email: Option<String>,
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

impl fmt::Display for CardsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(&self.preset)
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

/// Quick-and-dirty vCard preview: pulls the first `FN`, `EMAIL` and
/// `TEL` line out of the raw bytes for the cards listing. Avoids
/// parsing the whole vCard just to render three columns.
fn vcard_preview(bytes: &[u8]) -> (Option<String>, Option<String>, Option<String>) {
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return (None, None, None),
    };

    let mut fn_value = None;
    let mut email = None;
    let mut tel = None;

    for line in text.lines() {
        let line = line.trim();
        if fn_value.is_none() && line.starts_with("FN:") {
            fn_value = Some(line[3..].to_string());
        } else if email.is_none() {
            if let Some(rest) = pick_property(line, "EMAIL") {
                email = Some(rest);
            }
        } else if tel.is_none() {
            if let Some(rest) = pick_property(line, "TEL") {
                tel = Some(rest);
            }
        }
    }

    (fn_value, email, tel)
}

fn pick_property(line: &str, key: &str) -> Option<String> {
    if !line.starts_with(key) {
        return None;
    }

    let after_key = &line[key.len()..];
    let colon = after_key.find(':')?;
    Some(after_key[colon + 1..].to_string())
}
