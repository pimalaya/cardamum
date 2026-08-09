use core::fmt;

use comfy_table::{Cell, Color, Row, Table};
use io_jmap::rfc9610::{address_book::JmapAddressBook, contact_card::JmapContactCard};
use serde::Serialize;

use crate::shared::table::style_from_preset;

/// JSContact `name.full` of a card, or the empty string.
pub fn card_name(card: &JmapContactCard) -> &str {
    card.card
        .get("name")
        .and_then(|name| name.get("full"))
        .and_then(|full| full.as_str())
        .unwrap_or("")
}

/// Comma-joined ids of the address books a card belongs to.
pub fn card_books(card: &JmapContactCard) -> String {
    card.address_book_ids
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .join(",")
}

/// A list of address books; `--json` emits the raw JMAP objects and the
/// state token to feed to `changes`.
#[derive(Clone, Debug, Serialize)]
pub struct BooksReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "list")]
    pub books: Vec<JmapAddressBook>,
    pub state: String,
}

impl fmt::Display for BooksReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("DESC"),
                Cell::new("DEFAULT"),
            ]))
            .add_rows(self.books.iter().map(|book| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(book.id.as_deref().unwrap_or("")).fg(self.id_color))
                    .add_cell(Cell::new(book.name.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(book.description.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(book.is_default));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        writeln!(f, "state: {}", self.state)
    }
}

/// A single address book; `--json` emits the raw JMAP object.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct BookReport(pub JmapAddressBook);

impl fmt::Display for BookReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let book = &self.0;
        writeln!(f, "id: {}", book.id.as_deref().unwrap_or(""))?;
        writeln!(f, "name: {}", book.name.as_deref().unwrap_or(""))?;
        writeln!(
            f,
            "description: {}",
            book.description.as_deref().unwrap_or("")
        )?;
        writeln!(f, "is-default: {}", book.is_default)
    }
}

/// A list of contact cards; `--json` emits the raw JMAP objects and the
/// state token to feed to `changes`.
#[derive(Clone, Debug, Serialize)]
pub struct CardsReport {
    #[serde(skip)]
    pub preset: String,
    #[serde(skip)]
    pub id_color: Color,
    #[serde(rename = "list")]
    pub cards: Vec<JmapContactCard>,
    pub state: String,
}

impl fmt::Display for CardsReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("ADDRESS-BOOKS"),
            ]))
            .add_rows(self.cards.iter().map(|card| {
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(card.id.as_deref().unwrap_or("")).fg(self.id_color))
                    .add_cell(Cell::new(card_name(card)))
                    .add_cell(Cell::new(card_books(card)));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        writeln!(f, "state: {}", self.state)
    }
}

/// A single contact card; `--json` emits the raw JMAP object.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct CardReport(pub JmapContactCard);

impl CardReport {
    /// Whether the report has nothing to show: the card carries none of
    /// the three fields [`fmt::Display`] renders. A `set` response often
    /// lands here, carrying only server bookkeeping such as `updated`,
    /// which is the caller's cue to confirm in words instead.
    pub fn is_empty(&self) -> bool {
        let card = &self.0;
        card.id.is_none() && card_name(card).is_empty() && card_books(card).is_empty()
    }
}

impl fmt::Display for CardReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: a `set` response echoes only the properties the server
        // itself decided, so most of these are absent after a write.
        // Printing an empty slot for each would read as data loss.
        let card = &self.0;
        if let Some(id) = card.id.as_deref() {
            writeln!(f, "id: {id}")?;
        }

        let name = card_name(card);
        if !name.is_empty() {
            writeln!(f, "name: {name}")?;
        }

        let books = card_books(card);
        if !books.is_empty() {
            writeln!(f, "address-books: {books}")?;
        }

        Ok(())
    }
}

/// The result of a `/changes` call: created / updated / destroyed ids
/// since a state, plus the new state to sync from next.
#[derive(Clone, Debug, Serialize)]
pub struct ChangesReport {
    #[serde(skip)]
    pub preset: String,
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub destroyed: Vec<String>,
    pub new_state: String,
    pub has_more_changes: bool,
}

impl fmt::Display for ChangesReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([Cell::new("STATUS"), Cell::new("ID")]));
        for id in &self.created {
            table.add_row(Row::from([Cell::new("created"), Cell::new(id)]));
        }
        for id in &self.updated {
            table.add_row(Row::from([Cell::new("updated"), Cell::new(id)]));
        }
        for id in &self.destroyed {
            table.add_row(Row::from([Cell::new("destroyed"), Cell::new(id)]));
        }

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        writeln!(f, "new-state: {}", self.new_state)?;
        if self.has_more_changes {
            writeln!(f, "(more changes: run changes again from new-state)")?;
        }
        Ok(())
    }
}
