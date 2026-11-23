//! # JMAP renderers
//!
//! The output types of the `jmap` commands: plain-text lists and
//! `Label: value` lines on a terminal, the raw JMAP objects under
//! `--json`.

use core::fmt;

use io_jmap::rfc9610::{address_book::JmapAddressBook, contact_card::JmapContactCard};
use pimalaya_cli::{
    columns::Columns,
    error::ItemErrors,
    fields::Fields,
    records::{Record, Records},
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::error::ItemError;

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

/// A list of address books; `--json` emits the JMAP objects as the server
/// sent them and the state token to feed to `changes`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JmapAddressBooksOutput {
    /// The JMAP AddressBook objects, as the server sent them.
    #[serde(rename = "list")]
    pub books: Vec<serde_json::Value>,
    /// The state token to feed to `address-book changes`.
    pub state: String,
    /// The error of each id the get found no AddressBook for, coded
    /// `addressbook-not-found`, `[]` when none.
    pub errors: Vec<ItemError>,
}

impl fmt::Display for JmapAddressBooksOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut records = Records::default();

        for book in &self.books {
            let book: JmapAddressBook = serde_json::from_value(book.clone()).unwrap_or_default();
            let mut fields = Fields::default();
            fields.push("Name", text(book.name.as_deref()));

            let state = if book.is_default {
                vec![String::from("default")]
            } else {
                Vec::new()
            };

            records.push(Record {
                title: text(book.id.as_deref()),
                state,
                fields,
                body: text_lines(book.description.as_deref()),
            });
        }

        if !records.items.is_empty() {
            writeln!(f, "{records}")?;
        }

        writeln!(f, "state: {}", sanitize::text(&self.state))?;
        write_errors_after(f, &self.errors)
    }
}

/// Writes one blank line then the item errors, nothing when none.
fn write_errors_after(f: &mut fmt::Formatter<'_>, errors: &[ItemError]) -> fmt::Result {
    let mut rows = ItemErrors::default();

    for error in errors {
        rows.push(error);
    }

    rows.write_after(f)
}

/// A single address book; `--json` emits the raw JMAP object.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct JmapAddressBookOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub JmapAddressBook,
);

impl fmt::Display for JmapAddressBookOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let book = &self.0;
        let mut fields = Fields::default();
        fields.push("Id", text(book.id.as_deref()));
        fields.push("Name", text(book.name.as_deref()));
        fields.push("Description", text_lines(book.description.as_deref()));
        fields.push("Default", if book.is_default { "yes" } else { "no" });
        write!(f, "{fields}")
    }
}

/// A list of contact cards; `--json` emits the JMAP objects as the server
/// sent them, beside the get's `state`, to feed to `contact-card changes`,
/// or the query's `queryState`, to feed to a `queryChanges`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JmapContactCardsOutput {
    /// The JMAP ContactCard objects, as the server sent them.
    #[serde(rename = "list")]
    pub cards: Vec<serde_json::Value>,
    /// The state of the cards a get answered, to feed to `contact-card
    /// changes`; none for a query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// The state of the query a query answered, which a `queryChanges`
    /// takes and `contact-card changes` does not; none for a get.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_state: Option<String>,
    /// The error of each id a get found no ContactCard for, coded
    /// `card-not-found`, `[]` when none; none for a query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ItemError>>,
}

impl fmt::Display for JmapContactCardsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.cards.is_empty() {
            let mut columns = Columns::new(["ID", "ADDRESS-BOOKS", "NAME"]);

            for card in &self.cards {
                let card: JmapContactCard =
                    serde_json::from_value(card.clone()).unwrap_or_default();
                let card = &card;
                columns.push([
                    text(card.id.as_deref()),
                    sanitize::text(&card_books(card)).into_owned(),
                    sanitize::text(card_name(card)).into_owned(),
                ]);
            }

            writeln!(f, "{columns}")?;
        }

        if let Some(state) = &self.state {
            writeln!(f, "state: {}", sanitize::text(state))?;
        }

        if let Some(state) = &self.query_state {
            writeln!(f, "queryState: {}", sanitize::text(state))?;
        }

        write_errors_after(f, self.errors.as_deref().unwrap_or_default())
    }
}

/// A single contact card; `--json` emits the raw JMAP object.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct JmapContactCardOutput(
    #[schemars(with = "serde_json::Map<String, serde_json::Value>")] pub JmapContactCard,
);

impl JmapContactCardOutput {
    /// Whether the card carries nothing [`fmt::Display`] would render.
    ///
    /// A `set` response often lands here, carrying only server
    /// bookkeeping such as `updated`, which is the caller's cue to
    /// confirm in words instead.
    pub fn is_empty(&self) -> bool {
        let card = &self.0;
        card.id.is_none() && card_name(card).is_empty() && card_books(card).is_empty()
    }
}

impl fmt::Display for JmapContactCardOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: a `set` response echoes only the properties the server
        // itself decided, so most of these are absent after a write.
        // Printing an empty slot for each would read as data loss.
        let card = &self.0;
        let mut fields = Fields::default();

        if let Some(id) = card.id.as_deref() {
            fields.push("Id", sanitize::text(id));
        }

        let name = card_name(card);
        if !name.is_empty() {
            fields.push("Name", sanitize::text(name));
        }

        let books = card_books(card);
        if !books.is_empty() {
            fields.push("Address books", sanitize::text(&books));
        }

        write!(f, "{fields}")
    }
}

/// The ids a `/changes` call reports, plus the state to sync from next.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JmapChangesOutput {
    /// Ids created since the given state.
    pub created: Vec<String>,
    /// Ids updated since it.
    pub updated: Vec<String>,
    /// Ids destroyed since it.
    pub destroyed: Vec<String>,
    /// The state to sync from next.
    pub new_state: String,
    /// Whether the server held more changes back.
    pub has_more_changes: bool,
}

impl fmt::Display for JmapChangesOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let changes = [
            ("created", &self.created),
            ("updated", &self.updated),
            ("destroyed", &self.destroyed),
        ];

        if changes.iter().any(|(_, ids)| !ids.is_empty()) {
            let mut columns = Columns::new(["STATUS", "ID"]);

            for (status, ids) in changes {
                for id in ids {
                    columns.push([String::from(status), sanitize::text(id).into_owned()]);
                }
            }

            writeln!(f, "{columns}")?;
        }

        writeln!(f, "new-state: {}", sanitize::text(&self.new_state))?;
        if self.has_more_changes {
            writeln!(f, "(more changes: run changes again from new-state)")?;
        }
        Ok(())
    }
}

/// Renders an optional server value on one line, sanitized, or nothing.
fn text(value: Option<&str>) -> String {
    value.map(sanitize::text).unwrap_or_default().into_owned()
}

/// Renders an optional server text, sanitized line by line, or nothing.
fn text_lines(value: Option<&str>) -> String {
    let lines = value.unwrap_or_default().lines().map(sanitize::text);
    lines.collect::<Vec<_>>().join("\n")
}
