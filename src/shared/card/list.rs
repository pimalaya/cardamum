//! # Card list command
//!
//! Lists a page of cards, one line each under the account and the
//! addressbook, or as JSON, previewing the main vCard properties of each.

use std::fmt;

use anyhow::{Context, Result};
use clap::Parser;
use pimalaya_cli::{
    clap::args::PageTokenArg,
    listing::{Listing, ListingPage},
    printer::Printer,
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    backend::Backend,
    capability::Capability,
    error::{ItemError, partial_when},
    shared::{
        arg::AddressbookIdArg,
        card::{Card, project::CardFields},
        client::AddressbookClient,
    },
};

/// List vCards inside the given addressbook.
///
/// JSON output: `{"cards": [{"id", "addressbookId", "etag", "fnValue",
/// "email", "tel", "uid", "fullName", "givenName", "familyName",
/// "emails", "phones", "organization", "organizationUnits", "title",
/// "note"}], "nextPage", "errors": [{"id", "code", "error",
/// "sources"}]}`: the first `FN`, `EMAIL` and `TEL` as before, then the
/// common fields read through the decoded vCard, every address and
/// number included, then the cursor of the next page, `null` on the last
/// one, then the error of each card the page could not read, which
/// `cards` leaves out, and of a listing the server cut short (its `id`
/// `null`), `[]` when none, the command then exiting 3. A card whose body
/// is not available yet carries what its backend knows of it.
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
    /// to 25. Zero lists every card.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl CardListCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut AddressbookClient) -> Result<()> {
        client.require(&[Capability::CardList])?;
        let addressbook_id = client.account.addressbook_id(self.addressbook.id)?;
        let page = match &self.page_token.token {
            Some(token) => token
                .parse()
                .with_context(|| format!("Invalid page token {token}"))?,
            None => self.page,
        };
        let page = page.max(1);
        let page_size = self
            .page_size
            .unwrap_or(client.account.cards_list_page_size());
        let page_size = Some(page_size).filter(|size| *size > 0);
        let listing = client.list_cards(&addressbook_id, Some(page), page_size)?;

        let output = CardListOutput {
            account: client.name.clone(),
            backend: client.backend(),
            addressbook: addressbook_id,
            page: page_size.map(|_| page),
            page_size,
            cards: listing.cards.into_iter().map(CardRow::from).collect(),
            next_page: listing.more.then(|| (page + 1).to_string()),
            errors: listing.errors,
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// The `card list` output, the cards of one page and the cursor of the
/// next.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardListOutput {
    /// The name of the account listed.
    #[serde(skip)]
    pub account: String,
    /// The backend serving the account.
    #[serde(skip)]
    pub backend: Backend,
    /// The addressbook listed.
    #[serde(skip)]
    pub addressbook: String,
    /// The page listed, `None` when the listing is not paged.
    #[serde(skip)]
    pub page: Option<u32>,
    /// The size of a page, `None` when the listing is not paged.
    #[serde(skip)]
    pub page_size: Option<u32>,
    /// The cards of the requested page.
    pub cards: Vec<CardRow>,
    /// The cursor of the next page, `null` once the listing is complete,
    /// given back through `--page-token`.
    pub next_page: Option<String>,
    /// The error of each card the page could not read, in the order met,
    /// and of a listing the server cut short, its `id` `null`.
    pub errors: Vec<ItemError>,
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
    /// The common fields, read through the vCard's decoded model.
    #[serde(flatten)]
    pub fields: CardFields,
}

impl From<Card> for CardRow {
    fn from(card: Card) -> Self {
        let (fn_value, email, tel) = vcard_preview(&card.contents);
        Self {
            fields: CardFields::project(&card.contents),
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
        let backend = self.backend.to_string();
        let mut listing = Listing::new(&self.account, Some(&backend));
        listing.head("Addressbook", &self.addressbook);

        let page = match (self.page, self.page_size) {
            (Some(page), Some(size)) => ListingPage::Numbered {
                page: page.into(),
                size: size.into(),
            },
            _ => ListingPage::All,
        };

        let next = self.next_page.as_deref();
        listing.page(page, self.cards.len(), ["card", "cards"], next);

        // NOTE: a name is free text, so it reads last.
        for card in &self.cards {
            let cell = |value: &Option<String>| match value {
                Some(value) => sanitize::text(value).into_owned(),
                None => String::new(),
            };

            listing.items.push([
                sanitize::text(&card.id).into_owned(),
                cell(&card.email),
                cell(&card.tel),
                cell(&card.fn_value),
            ]);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
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

#[cfg(all(test, feature = "vdir"))]
mod tests {
    use super::*;

    fn card(id: &str, vcard: &str) -> CardRow {
        CardRow::from(Card {
            id: id.to_string(),
            addressbook_id: String::from("personal"),
            etag: None,
            contents: vcard.as_bytes().to_vec(),
        })
    }

    fn output() -> CardListOutput {
        CardListOutput {
            account: String::from("work"),
            backend: Backend::Vdir,
            addressbook: String::from("personal"),
            page: Some(1),
            page_size: Some(2),
            cards: vec![
                card(
                    "ada",
                    "BEGIN:VCARD\r\nFN:Ada Lovelace\r\nEMAIL:ada@example.org\r\n\
                     TEL;TYPE=cell:+33 6\r\nEND:VCARD\r\n",
                ),
                card("bob", "BEGIN:VCARD\r\nFN:Bob\r\nEND:VCARD\r\n"),
            ],
            next_page: Some(String::from("2")),
            errors: Vec::new(),
        }
    }

    /// A page prints its context and its paging, then one line per card
    /// with the name last, and nothing under the cards.
    #[test]
    fn a_page_prints_its_paging_above_one_line_per_card() {
        let expected = "\
Account:      work (vdir)
Addressbook:  personal
Page:         1, cards 1 to 2
Next page:    --page 2

ada  ada@example.org  +33 6  Ada Lovelace
bob  -                -      Bob
";
        assert_eq!(output().to_string(), expected);
    }

    #[test]
    fn the_last_page_says_so_and_an_empty_one_too() {
        let mut output = output();
        output.page = Some(2);
        output.next_page = None;
        output.cards.truncate(1);

        let expected = "\
Account:      work (vdir)
Addressbook:  personal
Page:         2, card 3, the last

ada  ada@example.org  +33 6  Ada Lovelace
";
        assert_eq!(output.to_string(), expected);

        output.page = Some(3);
        output.cards.clear();

        let expected = "\
Account:      work (vdir)
Addressbook:  personal
Page:         3, empty
";
        assert_eq!(output.to_string(), expected);
    }

    /// The errors print last, after the cards and one blank line, aligned
    /// among themselves, `-` for an absent id or code.
    #[test]
    fn the_errors_print_last_after_one_blank_line() {
        use anyhow::anyhow;

        use crate::{error, shared::client::unreadable};

        // NOTE: a card in `errors` is never among the cards, so no row of
        // empty cells stands for it.
        let mut output = output();
        output.errors = vec![
            error::unreadable_card(Some("bad"), unreadable(b"not a card").unwrap()),
            error::ItemError::new(Some("42"), &anyhow!("Gone")),
        ];

        let expected = "\
Account:      work (vdir)
Addressbook:  personal
Page:         1, cards 1 to 2
Next page:    --page 2

ada  ada@example.org  +33 6  Ada Lovelace
bob  -                -      Bob

Error:  bad  card-unreadable  Cannot parse the card body as a vCard
Error:  42   -                Gone
";
        assert_eq!(output.to_string(), expected);

        output.cards.clear();
        output.errors.truncate(1);
        let expected = "\
Account:      work (vdir)
Addressbook:  personal
Page:         1, empty
Next page:    --page 2

Error:  bad  card-unreadable  Cannot parse the card body as a vCard
";
        assert_eq!(output.to_string(), expected);
    }

    /// The text-only context stays out of the JSON, the cursor and the
    /// errors go in.
    #[test]
    fn the_json_carries_the_cards_the_cursor_and_the_errors() {
        let json = serde_json::to_value(output()).unwrap();
        let keys: Vec<_> = json.as_object().unwrap().keys().cloned().collect();
        assert_eq!(keys, ["cards", "errors", "nextPage"]);
        assert_eq!(json["nextPage"], "2");
        assert_eq!(json["errors"], serde_json::json!([]));
    }
}
