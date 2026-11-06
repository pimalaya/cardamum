//! # Pimdir index
//!
//! Read-only query client of the store's search index (pimdir SEARCH),
//! scoped to the account's cards.

use std::{
    fmt,
    io::{IsTerminal, stdout},
};

use anyhow::{Result, bail};
use ariadne::{Color, Config, Label, Report, ReportKind, Source};
use io_pimdir::{
    client::{
        reader::PimdirItem,
        search::{
            PimdirIndex, PimdirSearchClock, PimdirSearchHit, PimdirSearchMatched,
            PimdirSearchOptions,
        },
    },
    search::{
        coverage::{PimdirSearchCoverage, PimdirSearchTemporal},
        query::{
            PimdirQuery, PimdirQueryError, PimdirQueryErrorCode, PimdirQueryKind,
            PimdirQueryPredicate, PimdirQueryTerm,
        },
    },
    summary::{PimdirSummary, contact::PimdirContactSummary},
};
use pimalaya_cli::error::CodedError;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    error::ErrorCode,
    pimdir::{backend::PimdirBackend, client::refusal},
    shared::card::list::CardRow,
};

impl PimdirBackend {
    /// One page of the cards `query` matches in the account, or in the
    /// address book `addressbook` alone, ordered and paged by `options`.
    pub fn search_cards(
        &self,
        query: PimdirQuery,
        addressbook: Option<&str>,
        options: &PimdirSearchOptions,
    ) -> Result<PimdirCardPage> {
        let query = self.scope(query, addressbook)?;
        let index = self.index()?;
        let answer = index
            .search(&query, options)
            .map_err(|err| refusal(err, "Search the pimdir index"))?;

        let mut hits = Vec::with_capacity(answer.hits.len());
        for hit in answer.hits {
            hits.push(self.card_hit(&index, hit, addressbook)?);
        }

        Ok(PimdirCardPage {
            hits,
            more: answer.more,
            coverage: PimdirCardCoverage::from(&answer.coverage),
        })
    }

    /// How many cards `query` matches in the account, or in the address
    /// book `addressbook` alone.
    pub fn count_cards(
        &self,
        query: PimdirQuery,
        addressbook: Option<&str>,
    ) -> Result<(u64, PimdirCardCoverage)> {
        let query = self.scope(query, addressbook)?;
        let answer = self
            .index()?
            .count(&query, &PimdirSearchClock::default())
            .map_err(|err| refusal(err, "Count in the pimdir index"))?;

        Ok((answer.count, PimdirCardCoverage::from(&answer.coverage)))
    }

    /// Opens the store's search index for queries alone (io-pimdir
    /// `PimdirIndex::open_query`): read-only, never created, rebuilt nor
    /// refreshed.
    fn index(&self) -> Result<PimdirIndex> {
        let root = &self.inner.root;

        PimdirIndex::open_query(root).map_err(|err| {
            let context = format!("Open the pimdir search index of `{}`", root.display());
            refusal(err, context)
        })
    }

    /// `query` scoped to the account's cards, `addressbook` alone when
    /// given.
    ///
    /// NOTE: the scope stays top-level, where io-pimdir reads its coverage.
    fn scope(&self, query: PimdirQuery, addressbook: Option<&str>) -> Result<PimdirQuery> {
        let mut conjuncts = match query {
            PimdirQuery::And(conjuncts) => conjuncts,
            query => vec![query],
        };
        let term = |predicate| {
            PimdirQuery::Term(PimdirQueryTerm {
                predicate,
                start: 0,
                end: 0,
            })
        };

        conjuncts.push(term(PimdirQueryPredicate::Kind(PimdirQueryKind::Contact)));
        if let Some(account) = &self.inner.account {
            conjuncts.push(term(PimdirQueryPredicate::Account(account.clone())));
        }
        if let Some(addressbook) = addressbook {
            self.known_collection(addressbook)?;
            conjuncts.push(term(PimdirQueryPredicate::Collection(
                addressbook.to_string(),
            )));
        }

        Ok(PimdirQuery::And(conjuncts))
    }

    /// A hit as `card list` lists a card: read through the index's own
    /// reader, the queue not folded in, from the address book asked or
    /// else the first holding it.
    fn card_hit(
        &self,
        index: &PimdirIndex,
        hit: PimdirSearchHit,
        addressbook: Option<&str>,
    ) -> Result<PimdirCardHit> {
        let asked = hit
            .placements
            .iter()
            .find(|placement| Some(placement.collection.as_str()) == addressbook);
        let Some(placement) = asked.or(hit.placements.first()) else {
            bail!("Card `{}` has no address book in the search index", hit.seq);
        };

        let item = index
            .reader()
            .get_item(&placement.collection, hit.seq)
            .map_err(|err| refusal(err, format!("Read card `{}`", hit.seq)))?;
        // NOTE: a card dropped since the index read it keeps the index's title.
        let item = match item {
            Some(item) => item,
            None => PimdirItem {
                seq: hit.seq,
                link_id: placement.link_id.clone(),
                flags: placement.flags.clone(),
                sort_key: placement.sort_key.clone(),
                object: None,
                level: placement.level,
                summary: Some(PimdirSummary::Contact(PimdirContactSummary {
                    full_name: hit.title,
                    ..Default::default()
                })),
                retention: None,
            },
        };
        let card = self.card_from_item(&placement.collection, item)?;

        Ok(PimdirCardHit {
            card: CardRow::from(card),
            matched: hit.matched.map(PimdirCardMatched::from),
            snippet: hit.snippet,
        })
    }
}

/// Parses the words of a query, joined by spaces, as the pimdir SEARCH
/// query language (§8.3).
///
/// A query the language refuses (§8.4) is the error `query-refused`: in
/// plain output the fault is drawn over the query, in JSON its message
/// names the §8.4 code, the span in Unicode scalar values and what would
/// have been valid there.
pub fn parse_query(words: &[String], json: bool) -> Result<PimdirQuery> {
    let query = words.join(" ");

    PimdirQuery::parse(&query).map_err(|err| {
        let message = if json {
            refusal_line(&err)
        } else {
            refusal_report(&query, &err)
        };
        CodedError::new(ErrorCode::QueryRefused, message).into()
    })
}

/// A refusal as one line: its code, span and message, then what was
/// expected.
fn refusal_line(err: &PimdirQueryError) -> String {
    let mut line = format!(
        "Query refused, {} at {}..{}: {}",
        err.code.as_str(),
        err.start,
        err.end,
        err.message
    );

    if !err.expected.is_empty() {
        line.push_str(&format!("; expected {}", err.expected.join(", ")));
    }

    line
}

/// A refusal drawn over the query with ariadne, color dropped when stdout
/// is not a terminal.
fn refusal_report(query: &str, err: &PimdirQueryError) -> String {
    let name = "query";
    let config = Config::default().with_color(stdout().is_terminal());
    // NOTE: an empty span past the end is drawn on the last character.
    let end = err.end.max(err.start + 1);
    let mut report = Report::build(ReportKind::Error, (name, err.start..end))
        .with_config(config)
        .with_code(err.code.as_str())
        .with_message("Query refused")
        .with_label(
            Label::new((name, err.start..end))
                .with_message(&err.message)
                .with_color(Color::Red),
        );

    if !err.expected.is_empty() {
        report = report.with_note(format!("expected {}", err.expected.join(", ")));
    }
    if err.code == PimdirQueryErrorCode::UnknownField {
        report = report.with_help(format!("fields: {}", err.fields.join(", ")));
    }

    let mut buf = Vec::new();
    let _ = report.finish().write((name, Source::from(query)), &mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

/// One page of hits.
pub struct PimdirCardPage {
    /// The hits, in the order asked.
    pub hits: Vec<PimdirCardHit>,
    /// Whether hits follow this page.
    pub more: bool,
    /// What the answer could not see.
    pub coverage: PimdirCardCoverage,
}

/// A card the index matched.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirCardHit {
    /// The card as `card list` lists it; its `id` is what `card read`
    /// takes, with its `addressbookId`.
    #[serde(flatten)]
    pub card: CardRow,
    /// What a text term matched, `null` when only field terms did.
    pub matched: Option<PimdirCardMatched>,
    /// A passage of the card naming a query word, `null` without an
    /// indexed body.
    pub snippet: Option<String>,
}

/// What a text term matched (pimdir SEARCH §8.2).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PimdirCardMatched {
    /// The local body.
    Body,
    /// The summary of a card whose body is not local.
    Summary,
}

impl From<PimdirSearchMatched> for PimdirCardMatched {
    fn from(matched: PimdirSearchMatched) -> Self {
        match matched {
            PimdirSearchMatched::Body => Self::Body,
            PimdirSearchMatched::Summary => Self::Summary,
        }
    }
}

impl fmt::Display for PimdirCardMatched {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Body => "body",
            Self::Summary => "summary",
        })
    }
}

/// What an answer could not see (pimdir SEARCH §8.2), over the account's
/// cards.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirCardCoverage {
    /// Whether the answer is the whole answer: every other field at rest.
    pub complete: bool,
    /// Cards in scope whose body is not local, matched on their summary
    /// alone.
    pub bodiless: u64,
    /// Cards in scope whose last change the index has not folded in.
    pub unindexed: u64,
    /// How many changes the store drew since the index was refreshed,
    /// store-wide: the sync engine refreshes it.
    pub behind: u64,
    /// Whether a first sync is still listing an address book in scope.
    pub filling: bool,
}

impl From<&PimdirSearchCoverage> for PimdirCardCoverage {
    fn from(coverage: &PimdirSearchCoverage) -> Self {
        let filling = match &coverage.temporal {
            PimdirSearchTemporal::Full => false,
            PimdirSearchTemporal::Bounded { filling, .. } => *filling,
        };

        Self {
            complete: coverage.is_complete() && coverage.local.behind == 0,
            bodiless: coverage.local.bodiless.values().sum(),
            unindexed: coverage.local.unindexed,
            behind: coverage.local.behind,
            filling,
        }
    }
}

/// The line under a table, naming what the answer could not see; nothing
/// when it is complete.
impl fmt::Display for PimdirCardCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.complete {
            return Ok(());
        }

        let mut parts = Vec::new();

        if self.bodiless > 0 {
            parts.push(format!(
                "{} card(s) have no local body and matched on their summary alone",
                self.bodiless
            ));
        }
        if self.filling {
            parts.push(String::from(
                "a first sync is still listing an address book",
            ));
        }
        if self.unindexed > 0 {
            parts.push(format!(
                "{} card(s) changed since the index was refreshed",
                self.unindexed
            ));
        }
        if self.behind > 0 {
            parts.push(format!(
                "the index is {} change(s) behind the store, which the sync engine refreshes",
                self.behind
            ));
        }
        if parts.is_empty() {
            parts.push(String::from(
                "the store is not known to hold every card in scope",
            ));
        }

        writeln!(f, "Incomplete: {}", parts.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use io_pimdir::{
        change::PimdirWriteOp,
        client::{PimdirStore, search::PimdirIndexRefreshOptions},
        collection::{PimdirCheckpoint, PimdirCollectionId, PimdirScope},
        placement::{
            PimdirBase, PimdirFlags, PimdirHandle, PimdirLevel, PimdirLinkId, PimdirPlacement,
            PimdirStatus,
        },
        sql::search::FORMAT_VERSION,
        summary::{contact, mail::PimdirMailSummary},
    };
    use pimalaya_cli::error::code_of;
    use tempfile::TempDir;

    use crate::{config::PimdirConfig, pimdir::index::*};

    const BOOK: &str = "book";
    const TEAM: &str = "team";
    const INBOX: &str = "mail/INBOX";

    /// A vCard 4.0 of `uid` and `name`, then `lines`, each ending CRLF.
    fn card(uid: &str, name: &str, lines: &str) -> Vec<u8> {
        format!("BEGIN:VCARD\r\nVERSION:4.0\r\nUID:{uid}\r\nFN:{name}\r\n{lines}END:VCARD\r\n")
            .into_bytes()
    }

    /// Declares `collection` of the media type `kind`, synced by `source`
    /// and grouped under `account` when one is named, as a first sync
    /// finding it empty leaves it: a checkpoint and a closed round.
    fn synced(root: &Path, source: &str, account: Option<&str>, collection: &str, kind: &str) {
        let mut store = PimdirStore::open(root).unwrap().for_source(source);
        if let Some(account) = account {
            store = store.for_account(account);
        }
        store.ensure_collection(collection, kind).unwrap();
        store
            .write(vec![PimdirWriteOp::SetCheckpoint {
                collection: PimdirCollectionId::from(collection),
                checkpoint: PimdirCheckpoint(Vec::new()),
            }])
            .unwrap();
        store
            .write(vec![
                PimdirWriteOp::OpenRound {
                    collection: PimdirCollectionId::from(collection),
                    scope: PimdirScope::unbounded(),
                    band: false,
                },
                PimdirWriteOp::CloseRound {
                    collection: PimdirCollectionId::from(collection),
                    coverage: PimdirScope::unbounded(),
                    checkpoint: None,
                },
            ])
            .unwrap();
    }

    fn backend(root: &Path, account: Option<&str>) -> PimdirBackend {
        PimdirBackend::new(PimdirConfig {
            root: root.to_path_buf(),
            account: account.map(String::from),
        })
        .unwrap()
    }

    /// Applies the queue as the store's owner does.
    fn drain(root: &Path) {
        PimdirStore::open(root)
            .unwrap()
            .for_source("carddav")
            .drain()
            .unwrap();
    }

    /// Builds or refreshes the index, as the store's owner does.
    fn refresh(root: &Path) {
        PimdirIndex::open(root)
            .unwrap()
            .refresh(&PimdirIndexRefreshOptions::default())
            .unwrap();
    }

    /// Writes into `collection` a placement holding no body, as a listing
    /// leaves one before its body is fetched: `summary` alone, at `Meta`.
    fn bodiless(root: &Path, source: &str, collection: &str, handle: &str, summary: Placed) {
        let flags = PimdirFlags::default();
        PimdirStore::open(root)
            .unwrap()
            .for_source(source)
            .write(vec![PimdirWriteOp::UpsertPlacement(PimdirPlacement {
                collection: PimdirCollectionId::from(collection),
                handle: PimdirHandle(handle.into()),
                link_id: Some(summary.link_id),
                object: None,
                level: PimdirLevel::Meta,
                summary: Some(summary.summary),
                sort_key: summary.sort_key.into(),
                flags: flags.clone(),
                status: PimdirStatus::Clean,
                conflict_revision: None,
                conflict_object: None,
                base: Some(PimdirBase {
                    flags,
                    revision: None,
                    object: None,
                }),
                origin: None,
            })])
            .unwrap();
    }

    /// What a bodiless placement carries.
    struct Placed {
        link_id: PimdirLinkId,
        summary: PimdirSummary,
        sort_key: String,
    }

    /// A store whose `carddav` source syncs the address books `book` and
    /// `team` and whose `imap` source syncs an inbox, indexed: in `book`
    /// Carol, Alice and Bob, created in that order, and Erin, whose body
    /// was never fetched; in `team` Dave; in the inbox a mail whose body
    /// was never fetched. Carol's and Dave's notes and the mail's subject
    /// name the offsite.
    fn indexed_store() -> (TempDir, PimdirBackend) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for book in [BOOK, TEAM] {
            synced(root, "carddav", None, book, "text/vcard");
        }
        synced(root, "imap", None, INBOX, "message/rfc822");

        let mut backend = backend(root, None);
        let cards = [
            (
                BOOK,
                card(
                    "carol",
                    "Carol Roux",
                    "EMAIL:carol@example.org\r\nNOTE:Planning the offsite\r\n",
                ),
            ),
            (
                BOOK,
                card(
                    "alice",
                    "Alice Martin",
                    "EMAIL:alice@example.org\r\nTEL:06 12 34 56 78\r\n\
                     CATEGORIES:friends\r\nNOTE:Allergic to peanuts\r\n",
                ),
            ),
            (
                BOOK,
                card(
                    "bob",
                    "Bob Durand",
                    "EMAIL:bob@corp.example\r\nORG:Corp\r\n",
                ),
            ),
            (
                TEAM,
                card(
                    "dave",
                    "Dave Petit",
                    "EMAIL:dave@example.org\r\nNOTE:Hosting the offsite\r\n",
                ),
            ),
        ];
        for (book, body) in cards {
            backend.create_card(book, body).unwrap();
        }
        drain(root);

        let erin = contact::derive(&card("erin", "Erin Quill", "EMAIL:erin@example.org\r\n"));
        bodiless(
            root,
            "carddav",
            BOOK,
            "erin.vcf",
            Placed {
                link_id: erin.link_id,
                summary: erin.summary.unwrap(),
                sort_key: erin.sort_key.0,
            },
        );
        bodiless(
            root,
            "imap",
            INBOX,
            "1",
            Placed {
                link_id: PimdirLinkId("agenda@example.org".into()),
                summary: PimdirSummary::Mail(PimdirMailSummary {
                    message_id: Some("agenda@example.org".into()),
                    subject: "Offsite agenda".into(),
                    date: Some("2026-05-17T10:00:00Z".into()),
                    ..Default::default()
                }),
                sort_key: "2026-05-17T10:00:00Z".into(),
            },
        );
        refresh(root);

        (dir, backend)
    }

    fn query(text: &str) -> PimdirQuery {
        parse_query(&[text.to_string()], true).unwrap()
    }

    fn page(backend: &PimdirBackend, text: &str, addressbook: Option<&str>) -> PimdirCardPage {
        let options = PimdirSearchOptions {
            sort: io_pimdir::client::search::PimdirSearchSort::Name,
            ..Default::default()
        };
        backend
            .search_cards(query(text), addressbook, &options)
            .unwrap()
    }

    /// The display names of the hits of `text`, in name order.
    fn names(backend: &PimdirBackend, text: &str, addressbook: Option<&str>) -> Vec<String> {
        page(backend, text, addressbook)
            .hits
            .into_iter()
            .map(|hit| hit.card.fn_value.unwrap_or_default())
            .collect()
    }

    #[test]
    fn a_word_of_a_note_is_found_with_its_snippet() {
        let (_dir, backend) = indexed_store();

        let peanuts = page(&backend, "peanuts", None);
        assert_eq!(peanuts.hits.len(), 1);
        let hit = &peanuts.hits[0];
        assert_eq!(hit.card.fn_value.as_deref(), Some("Alice Martin"));
        assert_eq!(hit.card.addressbook_id, BOOK);
        assert_eq!(hit.card.fields.note.as_deref(), Some("Allergic to peanuts"));
        assert!(hit.card.etag.is_some());
        assert_eq!(hit.matched, Some(PimdirCardMatched::Body));
        assert!(
            hit.snippet.as_deref().unwrap().contains("peanuts"),
            "{:?}",
            hit.snippet
        );

        let json = serde_json::to_value(hit).unwrap();
        assert_eq!(json["id"], hit.card.id);
        assert_eq!(json["addressbookId"], BOOK);
        assert_eq!(json["fullName"], "Alice Martin");
        assert_eq!(json["matched"], "body");
    }

    #[test]
    fn a_phone_number_is_found_by_free_text() {
        let (_dir, backend) = indexed_store();

        assert_eq!(names(&backend, "0612345678", None), ["Alice Martin"]);
    }

    #[test]
    fn field_terms_reach_a_card() {
        let (_dir, backend) = indexed_store();

        assert_eq!(names(&backend, "has:phone", None), ["Alice Martin"]);
        assert_eq!(names(&backend, "category:friends", None), ["Alice Martin"]);
        assert_eq!(
            names(&backend, "email:@example.org", None),
            ["Alice Martin", "Carol Roux", "Dave Petit", "Erin Quill"]
        );
    }

    #[test]
    fn a_bodiless_card_matches_on_its_summary() {
        let (_dir, backend) = indexed_store();

        let quill = page(&backend, "quill", None);
        assert_eq!(quill.hits.len(), 1);
        let hit = &quill.hits[0];
        assert_eq!(hit.card.fn_value.as_deref(), Some("Erin Quill"));
        assert_eq!(hit.card.etag, None);
        assert_eq!(hit.matched, Some(PimdirCardMatched::Summary));

        assert_eq!(quill.coverage.bodiless, 1);
        assert!(!quill.coverage.complete);
        let line = quill.coverage.to_string();
        assert!(line.contains("1 card(s) have no local body"), "{line}");
    }

    /// `-k` narrows the hits to one address book and the coverage with
    /// them: Erin, bodiless, is in `book` alone.
    #[test]
    fn the_addressbook_narrows_the_hits_and_the_coverage() {
        let (_dir, backend) = indexed_store();

        assert_eq!(
            names(&backend, "offsite", None),
            ["Carol Roux", "Dave Petit"]
        );

        let team = page(&backend, "offsite", Some(TEAM));
        let names: Vec<_> = team.hits.iter().map(|hit| &hit.card.fn_value).collect();
        assert_eq!(names, [&Some(String::from("Dave Petit"))]);
        assert_eq!(team.hits[0].card.addressbook_id, TEAM);
        assert_eq!(team.coverage.bodiless, 0);
        assert!(team.coverage.complete, "{:?}", team.coverage);

        let (count, coverage) = backend.count_cards(query("offsite"), Some(BOOK)).unwrap();
        assert_eq!(count, 1);
        assert_eq!(coverage.bodiless, 1);

        let err = backend
            .search_cards(query("offsite"), Some("nope"), &Default::default())
            .err()
            .unwrap();
        assert!(err.to_string().contains("`nope` not found"), "{err}");
    }

    #[test]
    fn a_search_reads_cards_alone() {
        let (_dir, backend) = indexed_store();

        let (count, _) = backend.count_cards(query("agenda"), None).unwrap();
        assert_eq!(count, 0, "a mail is counted");
    }

    #[test]
    fn a_search_reads_the_configured_account_alone() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for account in ["work", "home"] {
            let book = format!("{account}/book");
            synced(root, "carddav", Some(account), &book, "text/vcard");
            backend(root, Some(account))
                .create_card(
                    &book,
                    card(account, "Jane Doe", "NOTE:Allergic to peanuts\r\n"),
                )
                .unwrap();
        }
        drain(root);
        refresh(root);

        for account in ["work", "home"] {
            let hits = page(&backend(root, Some(account)), "peanuts", None).hits;
            assert_eq!(hits.len(), 1, "{account}");
            assert_eq!(hits[0].card.addressbook_id, format!("{account}/book"));
        }
    }

    #[test]
    fn hits_come_in_name_order_and_pages() {
        let (_dir, backend) = indexed_store();
        let options = |offset| PimdirSearchOptions {
            sort: io_pimdir::client::search::PimdirSearchSort::Name,
            limit: 2,
            offset,
            ..Default::default()
        };

        assert_eq!(
            names(&backend, "kind:contact", None),
            [
                "Alice Martin",
                "Bob Durand",
                "Carol Roux",
                "Dave Petit",
                "Erin Quill"
            ]
        );

        let second = backend
            .search_cards(query("kind:contact"), None, &options(2))
            .unwrap();
        assert_eq!(second.hits.len(), 2);
        assert!(second.more);
        let last = backend
            .search_cards(query("kind:contact"), None, &options(4))
            .unwrap();
        assert_eq!(last.hits.len(), 1);
        assert!(!last.more);
    }

    #[test]
    fn a_queued_card_is_not_found_until_drained_and_refreshed() {
        let (dir, mut backend) = indexed_store();
        backend
            .create_card(BOOK, card("fred", "Fred Kayak", "NOTE:Paddles\r\n"))
            .unwrap();

        let paddles = page(&backend, "paddles", None);
        assert!(paddles.hits.is_empty(), "a queued create is listed");
        assert_eq!(paddles.coverage.behind, 0);

        drain(dir.path());
        assert!(page(&backend, "paddles", None).hits.is_empty());
        let (_, coverage) = backend.count_cards(query("kind:contact"), None).unwrap();
        assert!(coverage.behind > 0, "{coverage:?}");
        assert!(!coverage.complete);
        let line = coverage.to_string();
        assert!(line.contains("behind the store"), "{line}");
        assert!(line.contains("sync engine refreshes"), "{line}");

        refresh(dir.path());
        assert_eq!(names(&backend, "paddles", None), ["Fred Kayak"]);
    }

    /// The index answers from the store's live rows, so a card the owner
    /// removed since the last refresh lists no more, while the coverage
    /// says the index lags.
    #[test]
    fn a_card_removed_since_the_refresh_is_not_listed() {
        let (dir, mut backend) = indexed_store();
        let bob = page(&backend, "durand", None).hits[0].card.id.clone();
        backend.delete_card(BOOK, &bob, None).unwrap();
        drain(dir.path());

        let durand = page(&backend, "durand", None);
        assert!(durand.hits.is_empty());
        assert!(durand.coverage.behind > 0);
    }

    /// An address book whose first sync never closed a round may miss
    /// cards, which the coverage says.
    #[test]
    fn a_first_sync_still_listing_is_reported() {
        let (dir, backend) = indexed_store();
        let mut store = PimdirStore::open(dir.path()).unwrap().for_source("carddav");
        store.ensure_collection("new", "text/vcard").unwrap();
        store
            .write(vec![PimdirWriteOp::SetCheckpoint {
                collection: PimdirCollectionId::from("new"),
                checkpoint: PimdirCheckpoint(Vec::new()),
            }])
            .unwrap();
        drop(store);
        refresh(dir.path());

        let coverage = page(&backend, "peanuts", None).coverage;
        assert!(coverage.filling, "{coverage:?}");
        assert!(!coverage.complete);
        let line = coverage.to_string();
        assert!(line.contains("a first sync is still listing"), "{line}");
        assert!(page(&backend, "peanuts", Some(TEAM)).coverage.complete);
    }

    /// A query past the bounds of SEARCH §8.3 (1,024 characters, 32 groups
    /// open at once, 128 terms) is refused whole as `limit-exceeded`, in
    /// the ariadne drawing as in the JSON line.
    #[test]
    fn a_query_past_its_bounds_is_refused_as_limit_exceeded() {
        let long = "a".repeat(1025);
        let deep = format!("{}a{}", "(".repeat(33), ")".repeat(33));
        let wide = vec!["a"; 129].join(" ");

        for (text, span) in [(long, "1024..1025"), (deep, "32..33"), (wide, "256..257")] {
            for json in [true, false] {
                let err = parse_query(std::slice::from_ref(&text), json)
                    .err()
                    .unwrap();
                assert_eq!(code_of(&err), Some(ErrorCode::QueryRefused), "{err}");
                let err = err.to_string();
                assert!(err.contains("limit-exceeded"), "{err}");
                if json {
                    assert!(err.contains(&format!("limit-exceeded at {span}")), "{err}");
                }
            }
        }
    }

    #[test]
    fn a_refused_query_names_its_code_and_span() {
        let cases = [
            ("()", "empty-query"),
            ("nmae:alice", "unknown-field"),
            ("has:bogus", "invalid-value"),
            ("date:2026-13-45", "invalid-date"),
            ("\"alice", "unterminated-phrase"),
            ("(alice", "unbalanced-parenthesis"),
            ("alice and", "dangling-operator"),
            ("al*ice", "invalid-wildcard"),
        ];

        for (text, code) in cases {
            for json in [true, false] {
                let err = parse_query(&[text.to_string()], json).err().unwrap();
                assert_eq!(code_of(&err), Some(ErrorCode::QueryRefused), "{text}");
                assert!(err.to_string().contains(code), "{text}: {err}");
            }
        }

        let err = parse_query(&[String::from("nmae:alice")], true)
            .err()
            .unwrap();
        assert!(
            err.to_string()
                .starts_with("Query refused, unknown-field at 0..5"),
            "{err}"
        );
        let err = parse_query(&[String::from("nmae:alice")], false)
            .err()
            .unwrap();
        assert!(err.to_string().contains("fields: "), "{err}");
    }

    /// Asserts a search on `backend` fails with `code`, saying the sync
    /// engine builds the index and advising to delete nothing.
    fn assert_refused(backend: &PimdirBackend, code: ErrorCode) -> String {
        let err = backend
            .search_cards(query("peanuts"), None, &Default::default())
            .err()
            .unwrap();
        assert_eq!(code_of(&err), Some(code), "{err:#}");
        let err = format!("{err:#}");
        assert!(err.contains("sync engine"), "{err}");
        assert!(!err.contains("delete"), "{err}");
        err
    }

    #[test]
    fn a_missing_or_outdated_index_says_to_run_the_sync_engine() {
        let (dir, backend) = indexed_store();
        let (other, _) = indexed_store();

        // NOTE: an index another store built is one the owner rebuilds.
        for name in ["index.db", "index.db-wal", "index.db-shm"] {
            let _ = std::fs::remove_file(dir.path().join(name));
            let _ = std::fs::copy(other.path().join(name), dir.path().join(name));
        }
        assert_refused(&backend, ErrorCode::IndexOutdated);

        for name in ["index.db", "index.db-wal", "index.db-shm"] {
            let _ = std::fs::remove_file(dir.path().join(name));
        }
        assert_refused(&backend, ErrorCode::IndexMissing);
        assert!(!dir.path().join("index.db").exists(), "the index was built");
    }

    /// An index another draft of SEARCH built is refused as
    /// `index-version`, and left as it is.
    #[test]
    fn an_index_of_another_draft_is_refused_as_index_version() {
        let (dir, backend) = indexed_store();
        let path = dir.path().join("index.db");
        assert!(!dir.path().join("index.db-wal").exists());
        let mut db = std::fs::read(&path).unwrap();
        let (from, to) = (FORMAT_VERSION.as_bytes(), b"search-draft-08");
        let mut found = false;
        for at in 0..=db.len() - from.len() {
            if &db[at..at + from.len()] == from {
                db[at..at + from.len()].copy_from_slice(to);
                found = true;
            }
        }
        assert!(found, "no draft in the index");
        std::fs::write(&path, &db).unwrap();

        let err = assert_refused(&backend, ErrorCode::IndexVersion);
        assert!(err.contains("search-draft-08"), "{err}");
        assert_eq!(std::fs::read(&path).unwrap(), db, "the index was rebuilt");
    }

    #[test]
    fn a_complete_answer_prints_no_coverage_line() {
        let coverage = PimdirCardCoverage::from(&PimdirSearchCoverage::default());

        assert!(coverage.complete);
        assert_eq!(coverage.to_string(), "");
    }

    #[test]
    fn coverage_renders_as_camel_case_json() {
        let coverage = PimdirCardCoverage {
            complete: false,
            bodiless: 2,
            unindexed: 1,
            behind: 3,
            filling: true,
        };

        assert_eq!(
            serde_json::to_value(&coverage).unwrap(),
            serde_json::json!({
                "complete": false,
                "bodiless": 2,
                "unindexed": 1,
                "behind": 3,
                "filling": true,
            })
        );
        let line = coverage.to_string();
        for part in [
            "2 card(s) have no local body",
            "a first sync is still listing an address book",
            "1 card(s) changed since the index was refreshed",
            "the index is 3 change(s) behind the store",
        ] {
            assert!(line.contains(part), "{line}");
        }
    }
}
