//! # Pimdir search
//!
//! The `pimdir search` command, listing the account's cards a query
//! matches through the store's search index.

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use io_pimdir::client::search::{
    PimdirSearchClock, PimdirSearchCursor, PimdirSearchOptions, PimdirSearchSort,
};
use pimalaya_cli::{
    clap::args::PageTokenArg,
    listing::{Listing, ListingPage},
    printer::Printer,
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    account::context::Account,
    backend::Backend,
    error::{ItemError, page_token_invalid, partial_when},
    pimdir::{
        backend::PimdirBackend,
        index::{PimdirCardCoverage, PimdirCardHit, parse_query},
    },
};

/// Search the account's cards with the pimdir query language.
///
/// The query is pimdir SEARCH.md §8: words and "phrases" (a trailing `*`
/// for a prefix), `field:value` terms (`email:`, `has:phone`,
/// `has:address`, `has:birthday`, `has:photo`, `category:`, `collection:`
/// and the others), `and`, `or`, `not` or a leading `-` (its word after
/// `--`), and parentheses. Free text reaches a card's names,
/// organization, notes and numbers. It is read over the account's cards,
/// every address book of it unless `-k` names one.
///
/// The answer comes from the store's search index, which the sync engine
/// builds and refreshes after each run: a card is found once it is synced
/// and indexed, and a queued write shows in no answer. An `Incomplete:`
/// line above the list says what the answer could not see: cards whose
/// body is not local, the index lagging the store.
///
/// A page ends with the token of the place it stopped, which
/// `--page-token` takes back, so a card arriving or leaving in between
/// neither repeats nor hides another; the last page has none. A card the store cannot read is an
/// item error, the others listed.
///
/// JSON output: `{"hits": [{...card list row, "matched", "snippet"}],
/// "nextPage", "coverage": {"complete", "bodiless", "missing",
/// "unparseable", "unindexed", "behind", "filling"}, "errors": [{"id",
/// "code", "error", "sources"}]}`, the command exiting 3 when `errors` is
/// not empty.
#[derive(Debug, Parser)]
pub struct PimdirSearchCommand {
    /// Address book to search, every one of the account's when omitted.
    #[arg(short = 'k', long = "addressbook", value_name = "ADDRESSBOOK-ID")]
    pub addressbook: Option<String>,
    /// 1-indexed page number to fetch.
    #[arg(short, long, value_name = "N", default_value_t = 1)]
    pub page: usize,
    /// Maximum number of cards per page.
    ///
    /// Falls back to the `card.list.page-size` config when omitted, then
    /// to 25.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<usize>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
    /// The order of the cards.
    #[arg(long, value_enum, default_value_t)]
    pub sort: PimdirSearchSortArg,
    /// The query, its words joined by spaces.
    ///
    /// An option is read wherever it stands, before the query or after
    /// it. A word starting with `-`, a negated term, goes after `--`, the
    /// options before it: `pimdir search -s 5 -- alice -bob`.
    #[arg(value_name = "QUERY", required = true)]
    pub query: Vec<String>,
}

impl PimdirSearchCommand {
    /// Searches the index and prints one page of cards.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        name: &str,
        account: &Account,
        backend: &PimdirBackend,
    ) -> Result<()> {
        let words = self.query.join(" ");
        let query = parse_query(&self.query, printer.is_json())?;
        let limit = self
            .page_size
            .unwrap_or(account.cards_list_page_size() as usize)
            .max(1);
        let mut options = PimdirSearchOptions {
            sort: self.sort.into(),
            limit,
            after: None,
            clock: PimdirSearchClock::default(),
        };
        let addressbook = self.addressbook.as_deref();
        let mut past_end = None;

        let page = match &self.page_token.token {
            Some(token) => {
                let cursor = PimdirSearchCursor::from_token(token)
                    .ok_or_else(|| page_token_invalid(token))?;
                options.after = Some(cursor);
                None
            }
            None => {
                let page = self.page.max(1);

                for _ in 1..page {
                    let skipped = backend.search_cards(query.clone(), addressbook, &options)?;

                    match skipped.next.clone() {
                        Some(next) => options.after = Some(next),
                        None => {
                            past_end = Some(skipped);
                            break;
                        }
                    }
                }

                Some(page as u64)
            }
        };

        let found = match past_end {
            Some(mut last) => {
                last.hits.clear();
                last.errors.clear();
                last
            }
            None => backend.search_cards(query, addressbook, &options)?,
        };

        let output = PimdirSearchOutput {
            place: PimdirSearchPlace {
                account: name.to_string(),
                addressbook: self.addressbook,
                query: words,
                page,
                size: limit as u64,
            },
            hits: found.hits,
            next_page: found.next.as_ref().map(PimdirSearchCursor::token),
            coverage: found.coverage,
            errors: found.errors,
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// The order of a card search (pimdir SEARCH §8.2).
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum PimdirSearchSortArg {
    /// By display name, A to Z.
    #[default]
    Name,
    /// Best match of the query's words first.
    Relevance,
}

impl From<PimdirSearchSortArg> for PimdirSearchSort {
    fn from(sort: PimdirSearchSortArg) -> Self {
        match sort {
            PimdirSearchSortArg::Name => Self::Name,
            PimdirSearchSortArg::Relevance => Self::Relevance,
        }
    }
}

/// The `pimdir search` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirSearchOutput {
    /// Where the page stands.
    #[serde(skip)]
    pub place: PimdirSearchPlace,
    /// The cards of this page.
    pub hits: Vec<PimdirCardHit>,
    /// The number of the next page, `null` once the answer is complete,
    /// given back through `--page-token`.
    pub next_page: Option<String>,
    /// What the answer could not see.
    pub coverage: PimdirCardCoverage,
    /// The error of each card the store could not read, in the order met,
    /// left out of `hits`.
    pub errors: Vec<ItemError>,
}

impl fmt::Display for PimdirSearchOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let place = &self.place;
        let backend = Backend::Pimdir.to_string();
        let mut listing = Listing::new(&place.account, Some(&backend));

        if let Some(addressbook) = &place.addressbook {
            listing.head("Addressbook", addressbook);
        }

        listing.head("Query", &place.query);

        let page = match place.page {
            Some(page) => ListingPage::Numbered {
                page,
                size: place.size,
            },
            None => ListingPage::Token,
        };
        let numbered_next = place.page.map(|page| (page + 1).to_string());
        let next = match place.page {
            Some(_) => self.next_page.as_ref().and(numbered_next.as_deref()),
            None => self.next_page.as_deref(),
        };
        listing.page(page, self.hits.len(), ["card", "cards"], next);
        self.coverage.note(&mut listing);

        let matched = self.hits.iter().any(|hit| hit.matched.is_some());

        // NOTE: a name is free text, so it reads last, as in `card list`.
        for hit in &self.hits {
            let [id, email, phone, name] = hit.card.cells();
            let mut line = vec![id, sanitize::text(&hit.card.addressbook_id).into_owned()];

            if matched {
                let matched = hit.matched.map(|matched| matched.to_string());
                line.push(matched.unwrap_or_default());
            }

            line.extend([email, phone, name]);
            listing.items.push(line);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
    }
}

/// Where a page of `pimdir search` stands: the account, the address book,
/// the query and the page.
#[derive(Clone, Debug, Default)]
pub struct PimdirSearchPlace {
    /// The name of the account searched.
    pub account: String,
    /// The address book `-k` narrowed the search to.
    pub addressbook: Option<String>,
    /// The query, its words joined by spaces.
    pub query: String,
    /// The page listed by number, from 1; `None` for a page `--page-token`
    /// reached.
    pub page: Option<u64>,
    /// The size of a page.
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use crate::{
        error::{ErrorCode, ItemError},
        pimdir::{
            index::{PimdirCardCoverage, PimdirCardHit, PimdirCardMatched},
            search::{PimdirSearchOutput, PimdirSearchPlace},
        },
        shared::card::{Card, list::CardRow},
    };

    /// One hit, on the last page of an incomplete answer.
    fn output() -> PimdirSearchOutput {
        let card = Card {
            id: String::from("7"),
            addressbook_id: String::from("book"),
            etag: None,
            contents: b"BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Alice Martin\r\n\
                EMAIL:alice@example.org\r\nEND:VCARD\r\n"
                .to_vec(),
        };

        PimdirSearchOutput {
            place: PimdirSearchPlace {
                account: String::from("work"),
                addressbook: None,
                query: String::from("peanuts"),
                page: Some(1),
                size: 25,
            },
            hits: vec![PimdirCardHit {
                card: CardRow::from(card),
                matched: Some(PimdirCardMatched::Body),
                snippet: Some(String::from("Allergic to peanuts")),
            }],
            next_page: None,
            coverage: PimdirCardCoverage {
                bodiless: 1,
                ..Default::default()
            },
            errors: Vec::new(),
        }
    }

    /// A hit prints on one line, its address book and what matched before
    /// its email, phone and name, under the account, the query, the page
    /// and what the answer could not see.
    #[test]
    fn a_hit_prints_on_one_line() {
        let expected = "\
Account:     work (pimdir)
Query:       peanuts
Page:        1, card 1, the last
Incomplete:  1 card(s) have no local body and matched on their summary alone

7  book  body  alice@example.org  -  Alice Martin
";
        assert_eq!(output().to_string(), expected);
    }

    /// A page pages as every listing does: `nextPage`, which `--page-token`
    /// takes, `--page` in text, and no `more`; a card the store could not
    /// read is an `errors` entry, printed after the hits.
    #[test]
    fn a_page_names_the_next_and_its_errors() {
        let mut output = output();
        output.place.addressbook = Some(String::from("book"));
        output.coverage.complete = true;
        output.next_page = Some(String::from("2"));
        output.errors = vec![ItemError {
            id: Some(String::from("9")),
            code: Some(ErrorCode::CardUnreadable),
            error: String::from("Cannot read the card body"),
            sources: Vec::new(),
        }];

        let json = serde_json::to_value(&output).unwrap();
        let mut keys: Vec<&str> = json.as_object().unwrap().keys().map(|k| &**k).collect();
        keys.sort();
        assert_eq!(keys, ["coverage", "errors", "hits", "nextPage"]);
        assert_eq!(json["nextPage"], "2");
        assert_eq!(json["errors"][0]["id"], "9");
        assert_eq!(json["errors"][0]["code"], "card-unreadable");

        let expected = "\
Account:      work (pimdir)
Addressbook:  book
Query:        peanuts
Page:         1, card 1

7  book  body  alice@example.org  -  Alice Martin

Next page: --page 2

Error:  9  card-unreadable  Cannot read the card body
";
        assert_eq!(output.to_string(), expected);
    }
}
