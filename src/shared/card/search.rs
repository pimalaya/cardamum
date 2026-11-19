//! # Card search command
//!
//! Lists a page of the account's cards a pimdir SEARCH query matches,
//! answered by the store's index on pimdir and by matching every card of
//! the scope elsewhere, so a query answers one hit set on every backend.

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use io_pimdir::search::{
    card::PimdirCard,
    fold,
    query::{PimdirQuery, PimdirQueryTerm},
};
use pimalaya_cli::{
    clap::args::PageTokenArg,
    error::CodedError,
    listing::{Listing, ListingPage},
    printer::Printer,
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    backend::Backend,
    capability::Capability,
    error::{ErrorCode, ItemError, page_token_invalid, partial_when},
    shared::{card::list::CardRow, client::AddressbookClient, query::parse_query},
};

/// Search the account's cards with the pimdir query language.
///
/// The query is pimdir SEARCH.md §8, the one `pimdir search` takes: words
/// and "phrases" (a trailing `*` for a prefix), `field:value` terms
/// (`email:`, `has:phone`, `category:` and the others), `and`, `or`,
/// `not` or a leading `-` (its word after `--`), and parentheses. It is
/// parsed before any backend is reached, so a query is refused the same
/// way everywhere (`query-refused`). It reads every address book of the
/// account, or the one `-k` names.
///
/// pimdir answers from the store's index, as `pimdir search` does. vdir
/// reads every card of the address books in scope and matches each as an
/// index holding them would: a term only a pimdir store answers (`tag:`,
/// `flag:`, `is:`, `changed:`, `thread:`, `id:`, `linked:`, `account:`,
/// `collection:`) is refused there whole, before anything is read
/// (`query-unsupported`).
///
/// Hits are ordered by name, or by relevance on a backend ranking them
/// (`card.search.relevance`).
///
/// JSON output: `{"cards": [{...card list row}], "nextPage",
/// "complete", "errors": [{"id", "code", "error", "sources"}]}`:
/// `complete` is `false` when the index could not see every card of the
/// scope (`pimdir search` says what it missed), and `errors` lists the
/// cards that could not be read, the command then exiting 3.
#[derive(Debug, Parser)]
pub struct CardSearchCommand {
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
    pub sort: CardSearchSort,
    /// The query, its words joined by spaces.
    ///
    /// A word starting with `-`, a negated term, goes after `--`, the
    /// options before it: `card search -s 5 -- alice -bob`.
    #[arg(value_name = "QUERY", required = true)]
    pub query: Vec<String>,
}

impl CardSearchCommand {
    /// Searches the cards in scope and prints one page of hits.
    pub fn execute(self, printer: &mut impl Printer, client: &mut AddressbookClient) -> Result<()> {
        let words = self.query.join(" ");
        let query = parse_query(&self.query, printer.is_json())?;

        client.require(&[Capability::CardSearch])?;

        let limit = self
            .page_size
            .unwrap_or(client.account.cards_list_page_size() as usize)
            .max(1);
        let search = CardSearch {
            query,
            words: words.clone(),
            addressbook: self.addressbook.clone(),
            sort: self.sort,
            limit,
            page: self.page.max(1),
            token: self.page_token.token,
        };
        let found = client.search_cards(&search)?;

        let output = CardSearchOutput {
            account: client.name.clone(),
            backend: client.backend(),
            addressbook: self.addressbook,
            query: words,
            page: found.page,
            size: limit as u64,
            cards: found.cards,
            next_page: found.next_page,
            complete: found.complete,
            errors: found.errors,
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }
}

/// The order of a card search (pimdir SEARCH §8.2).
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum CardSearchSort {
    /// By display name, A to Z.
    #[default]
    Name,
    /// Best match of the query's words first, where the backend ranks.
    Relevance,
}

/// A search as the command asks it of a backend.
pub struct CardSearch {
    /// The parsed query.
    pub query: PimdirQuery,
    /// The query as written, its words joined by spaces.
    pub words: String,
    /// The address book searched, every one of the account's when `None`.
    pub addressbook: Option<String>,
    /// The order of the hits.
    pub sort: CardSearchSort,
    /// The size of a page, 1 or more.
    pub limit: usize,
    /// The page asked by number, from 1, when no token is given.
    pub page: usize,
    /// The `nextPage` a previous page printed.
    pub token: Option<String>,
}

/// One page of hits, as a backend answers it.
#[derive(Debug, Default)]
pub struct CardSearchPage {
    /// The cards of the page.
    pub cards: Vec<CardRow>,
    /// The page listed by number, `None` for one a token reached.
    pub page: Option<u64>,
    /// The token of the next page, `None` on the last one.
    pub next_page: Option<String>,
    /// Whether the answer saw every card of the scope.
    pub complete: bool,
    /// The error of each card that could not be read.
    pub errors: Vec<ItemError>,
}

/// The refusal of a term only a pimdir store answers, on a backend
/// holding none: answering without it would widen the hits silently.
pub fn check_store_terms(search: &CardSearch, backend: Backend) -> Result<()> {
    let Some(term) = search.query.store_terms().into_iter().next() else {
        return Ok(());
    };

    let message = format!(
        "Query term `{}` at {}..{} reads a pimdir store, which the {backend} backend has none \
         of: nothing was searched",
        term_text(&search.words, term),
        term.start,
        term.end,
    );
    Err(CodedError::new(ErrorCode::QueryUnsupported, message).into())
}

/// The text of `term` in `words`, its span counted in Unicode scalar
/// values.
fn term_text(words: &str, term: &PimdirQueryTerm) -> String {
    words
        .chars()
        .skip(term.start)
        .take(term.end.saturating_sub(term.start))
        .collect()
}

/// The page of `search` answered by matching every card of its scope,
/// read through `client`, with io-pimdir's matcher (SEARCH §8.5), so the
/// hits are the ones an index holding those cards answers.
///
/// Ordered by the folded full name (SEARCH §8.2, STORAGE Annex A.2), then
/// by address book and id, no matcher ranking its hits: relevance is
/// refused, as the store terms are, before anything is read. A page is
/// numbered, its token the number of the next. The cards that could not
/// be read are listed on the first page.
pub fn matched_page(client: &mut AddressbookClient, search: &CardSearch) -> Result<CardSearchPage> {
    if let CardSearchSort::Relevance = search.sort {
        client.require(&[Capability::CardSearchRelevance])?;
    }
    check_store_terms(search, client.backend())?;

    let page = match &search.token {
        Some(token) => token
            .parse::<usize>()
            .ok()
            .filter(|page| *page > 0)
            .ok_or_else(|| page_token_invalid(token))?,
        None => search.page,
    };

    let books = client.list_addressbooks()?;
    let scope: Vec<String> = match &search.addressbook {
        Some(id) => {
            if !books.iter().any(|book| &book.id == id) {
                let held: Vec<&str> = books.iter().map(|book| book.id.as_str()).collect();
                let message = format!(
                    "Address book `{id}` not found; the account holds: {}",
                    held.join(", ")
                );
                return Err(CodedError::new(ErrorCode::AddressbookNotFound, message).into());
            }
            vec![id.clone()]
        }
        None => books.into_iter().map(|book| book.id).collect(),
    };

    let mut cards = Vec::new();
    let mut errors = Vec::new();

    for id in &scope {
        let listing = client.list_cards(id, None, None)?;
        cards.extend(listing.cards);
        errors.extend(listing.errors);
    }

    let read: Vec<PimdirCard> = cards
        .iter()
        .map(|card| PimdirCard::read(&card.contents))
        .collect();
    let matched = search.query.match_cards(&read).map_err(|err| {
        let message = format!("{err}: nothing was searched");
        CodedError::new(ErrorCode::QueryUnsupported, message)
    })?;

    let mut hits: Vec<(String, CardRow)> = cards
        .into_iter()
        .zip(matched)
        .filter_map(|(card, matched)| matched.then(|| CardRow::from(card)))
        .map(|row| {
            let key = fold::value(row.fields.full_name.as_deref().unwrap_or_default());
            (key, row)
        })
        .collect();
    hits.sort_by(|(a, x), (b, y)| {
        a.cmp(b)
            .then_with(|| x.addressbook_id.cmp(&y.addressbook_id))
            .then_with(|| x.id.cmp(&y.id))
    });

    let skip = (page - 1).saturating_mul(search.limit);
    let more = hits.len() > skip.saturating_add(search.limit);
    let cards = hits
        .into_iter()
        .skip(skip)
        .take(search.limit)
        .map(|(_, row)| row)
        .collect();

    Ok(CardSearchPage {
        cards,
        page: Some(page as u64),
        next_page: more.then(|| (page + 1).to_string()),
        complete: true,
        errors: match page {
            1 => errors,
            _ => Vec::new(),
        },
    })
}

/// The `card search` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardSearchOutput {
    /// The name of the account searched.
    #[serde(skip)]
    pub account: String,
    /// The backend serving the account.
    #[serde(skip)]
    pub backend: Backend,
    /// The address book `-k` narrowed the search to.
    #[serde(skip)]
    pub addressbook: Option<String>,
    /// The query, its words joined by spaces.
    #[serde(skip)]
    pub query: String,
    /// The page listed by number, `None` for one a token reached.
    #[serde(skip)]
    pub page: Option<u64>,
    /// The size of a page.
    #[serde(skip)]
    pub size: u64,
    /// The cards of the page, each its `card list` row.
    pub cards: Vec<CardRow>,
    /// The token of the next page, `null` once the answer is complete,
    /// given back through `--page-token`.
    pub next_page: Option<String>,
    /// Whether the answer saw every card of the scope: `false` when the
    /// pimdir index lags the store or misses a body.
    pub complete: bool,
    /// The error of each card that could not be read, in the order met.
    pub errors: Vec<ItemError>,
}

impl fmt::Display for CardSearchOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let backend = self.backend.to_string();
        let mut listing = Listing::new(&self.account, Some(&backend));

        if let Some(addressbook) = &self.addressbook {
            listing.head("Addressbook", addressbook);
        }

        listing.head("Query", &self.query);

        let page = match self.page {
            Some(page) => ListingPage::Numbered {
                page,
                size: self.size,
            },
            None => ListingPage::Token,
        };
        let numbered_next = self.page.map(|page| (page + 1).to_string());
        let next = match self.page {
            Some(_) => self.next_page.as_ref().and(numbered_next.as_deref()),
            None => self.next_page.as_deref(),
        };
        listing.page(page, self.cards.len(), ["card", "cards"], next);

        if !self.complete {
            listing.head(
                "Incomplete",
                "the index could not see every card; `pimdir search` says what it missed",
            );
        }

        for card in &self.cards {
            let [id, email, phone, name] = card.cells();
            let book = sanitize::text(&card.addressbook_id).into_owned();
            listing.items.push(vec![id, book, email, phone, name]);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
    }
}

/// The page of `search` the pimdir store's index answers, as `pimdir
/// search` pages it: its token is the index's cursor, and `complete` its
/// coverage verdict.
#[cfg(feature = "pimdir")]
pub fn pimdir_page(
    backend: &crate::pimdir::backend::PimdirBackend,
    search: &CardSearch,
) -> Result<CardSearchPage> {
    use io_pimdir::client::search::{PimdirSearchCursor, PimdirSearchSort};

    let sort = match search.sort {
        CardSearchSort::Name => PimdirSearchSort::Name,
        CardSearchSort::Relevance => PimdirSearchSort::Relevance,
    };
    let (found, page) = crate::pimdir::search::search_page(
        backend,
        search.query.clone(),
        search.addressbook.as_deref(),
        sort,
        search.limit,
        search.page,
        search.token.as_deref(),
    )?;

    Ok(CardSearchPage {
        cards: found.hits.into_iter().map(|hit| hit.card).collect(),
        page,
        next_page: found.next.as_ref().map(PimdirSearchCursor::token),
        complete: found.coverage.complete,
        errors: found.errors,
    })
}

#[cfg(all(test, feature = "msgraph"))]
mod tests {
    use io_msgraph::v1::client::MsgraphClientStd;
    use io_pimdir::search::query::PimdirQuery;
    use serde_json::json;

    use crate::{
        backend::Backend,
        config::{AccountConfig, Config},
        msgraph::backend::MsgraphBackend,
        script::{self, Script},
        shared::{
            card::search::{CardSearch, CardSearchSort, matched_page},
            client::AddressbookClient,
        },
    };

    /// A Graph answer listing `value`, one page.
    fn page(value: serde_json::Value) -> Vec<u8> {
        let body = json!({ "value": value }).to_string();
        script::response(200, "application/json", &body)
    }

    /// A search of `query` over every address book of the account.
    fn search(query: &str) -> CardSearch {
        CardSearch {
            query: PimdirQuery::parse(query).unwrap(),
            words: query.to_owned(),
            addressbook: None,
            sort: CardSearchSort::Name,
            limit: 25,
            page: 1,
            token: None,
        }
    }

    /// On Graph, a search lists every folder whatever its depth, every
    /// card of each, and matches them as an index would: a diacritic-free
    /// word finds the accented name of a card two folders deep.
    #[test]
    fn a_graph_search_matches_every_folder_depth() {
        let (script, _) = Script::new([
            page(json!([{"id": "AAMkW", "displayName": "Work"}])),
            page(json!([{"id": "AAMkP", "displayName": "Projects"}])),
            page(json!([])),
            page(json!([{"id": "AAMkA", "displayName": "Ada Byron"}])),
            page(json!([{"id": "AAMkB", "displayName": "Bob Martin"}])),
            page(json!([{"id": "AAMkE", "displayName": "Élodie Martin"}])),
        ]);
        let account = AccountConfig {
            msgraph: Some(toml::from_str(r#"auth.token.raw = "token""#).unwrap()),
            ..Default::default()
        };
        let mut client =
            AddressbookClient::new(Config::default(), "work".into(), account, Backend::Auto)
                .unwrap();
        client.open_msgraph(MsgraphBackend {
            inner: MsgraphClientStd::new(script, "token", Default::default()),
        });

        let found = matched_page(&mut client, &search("elodie")).unwrap();

        let hits: Vec<_> = found
            .cards
            .iter()
            .map(|card| (&*card.addressbook_id, &*card.id))
            .collect();
        assert_eq!(hits, [("AAMkP", "AAMkE")]);
        assert!(found.complete);
        assert_eq!(found.next_page, None);
    }
}
