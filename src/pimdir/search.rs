//! # Pimdir search
//!
//! The `pimdir search` command, listing the account's cards a query
//! matches through the store's search index.

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use io_pimdir::client::search::{PimdirSearchClock, PimdirSearchOptions, PimdirSearchSort};
use pimalaya_cli::{
    printer::Printer,
    table::{Cell, Color, ColumnConstraint, ContentArrangement, Row, Table, sanitize},
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    account::context::Account,
    pimdir::{
        backend::PimdirBackend,
        index::{PimdirCardCoverage, PimdirCardHit, parse_query},
    },
    shared::table::style_from_preset,
};

/// Search the account's cards with the pimdir query language.
///
/// The query is pimdir SEARCH.md §8: words and "phrases" (a trailing `*`
/// for a prefix), `field:value` terms (`email:`, `has:phone`,
/// `has:address`, `has:birthday`, `has:photo`, `category:`, `collection:`
/// and the others), `and`, `or`, `not` or a leading `-`, and parentheses.
/// Free text reaches a card's names, organization, notes and numbers. It
/// is read over the account's cards, every address book of it unless
/// `-k` names one.
///
/// The answer comes from the store's search index, which the sync engine
/// builds and refreshes after each run: a card is found once it is synced
/// and indexed, and a queued write shows in no answer. A line under the
/// table says what the answer could not see: cards whose body is not
/// local, the index lagging the store.
///
/// JSON output: `{"hits": [{...card list row, "matched", "snippet"}],
/// "more", "coverage": {"complete", "bodiless", "unindexed", "behind",
/// "filling"}}`.
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
    /// The order of the cards.
    #[arg(long, value_enum, default_value_t)]
    pub sort: PimdirSearchSortArg,
    /// The query, its words joined by spaces.
    #[arg(value_name = "QUERY", required = true)]
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub query: Vec<String>,
}

impl PimdirSearchCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        account: &Account,
        backend: &PimdirBackend,
    ) -> Result<()> {
        let query = parse_query(&self.query, printer.is_json())?;
        let limit = self
            .page_size
            .unwrap_or(account.cards_list_page_size() as usize)
            .max(1);
        let options = PimdirSearchOptions {
            sort: self.sort.into(),
            limit,
            offset: self.page.saturating_sub(1).saturating_mul(limit),
            clock: PimdirSearchClock::default(),
        };
        let page = backend.search_cards(query, self.addressbook.as_deref(), &options)?;

        printer.out(PimdirSearchOutput {
            preset: account.table_preset().to_string(),
            arrangement: account.table_arrangement(),
            id_color: account.cards_list_table_id_color(),
            fn_color: account.cards_list_table_fn_color(),
            email_color: account.cards_list_table_email_color(),
            tel_color: account.cards_list_table_tel_color(),
            hits: page.hits,
            more: page.more,
            coverage: page.coverage,
        })
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
    /// The `comfy_table` preset the table is drawn with.
    #[serde(skip)]
    pub preset: String,
    /// How the table arranges its content.
    #[serde(skip)]
    pub arrangement: ContentArrangement,
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
    /// The cards of this page.
    pub hits: Vec<PimdirCardHit>,
    /// Whether cards follow this page.
    pub more: bool,
    /// What the answer could not see.
    pub coverage: PimdirCardCoverage,
}

impl fmt::Display for PimdirSearchOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_content_arrangement(self.arrangement.clone())
            .set_header(Row::from([
                "ID",
                "FN",
                "EMAIL",
                "TEL",
                "ADDRESSBOOK",
                "MATCHED",
                "SNIPPET",
            ]))
            .add_rows(self.hits.iter().map(|hit| {
                let card = &hit.card;
                let text =
                    |value: &Option<String>| sanitize(value.as_deref().unwrap_or("")).into_owned();
                let matched = match hit.matched {
                    Some(matched) => matched.to_string(),
                    None => String::new(),
                };
                let snippet = hit
                    .snippet
                    .as_deref()
                    .unwrap_or_default()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");

                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(sanitize(&card.id)).fg(self.id_color))
                    .add_cell(Cell::new(text(&card.fn_value)).fg(self.fn_color))
                    .add_cell(Cell::new(text(&card.email)).fg(self.email_color))
                    .add_cell(Cell::new(text(&card.tel)).fg(self.tel_color))
                    .add_cell(Cell::new(sanitize(&card.addressbook_id)))
                    .add_cell(Cell::new(matched))
                    .add_cell(Cell::new(sanitize(&snippet)));
                row
            }));

        if let Some(column) = table.column_mut(0) {
            column.set_constraint(ColumnConstraint::ContentWidth);
        }

        writeln!(f)?;
        writeln!(f, "{table}")?;
        write!(f, "{}", self.coverage)
    }
}
