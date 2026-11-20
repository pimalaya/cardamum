//! # ContactCard query command
//!
//! Searches the ContactCards of the account, then fetches the ones the
//! search matched.

use anyhow::Result;
use clap::Parser;
use io_jmap::rfc9610::contact_card::query::{JmapContactCardFilter, JmapContactCardQueryOptions};
use pimalaya_cli::{clap::args::PageTokenArg, output::Paginated, printer::Printer};

use crate::{
    error::page_token_invalid,
    jmap::{client::JmapClient, render::JmapContactCardsOutput},
};

/// Query JMAP ContactCards (`ContactCard/query` + `/get`).
///
/// JMAP pages by position: pass the `nextPage` a page printed to
/// `--page-token` for the next one, with the same other flags.
///
/// JSON output: `{"list": [<raw JMAP ContactCard>...], "queryState",
/// "nextPage"}`, `nextPage` the position the next page starts at, `null`
/// on the last page.
#[derive(Debug, Parser)]
pub struct JmapContactCardQueryCommand {
    /// Restrict to cards in this AddressBook.
    #[arg(short = 'k', long, value_name = "ADDRESS-BOOK")]
    pub in_address_book: Option<String>,
    /// Free-text match against any text in the card.
    #[arg(long, value_name = "TEXT")]
    pub text: Option<String>,
    /// Maximum number of cards to return.
    #[arg(short = 'l', long, value_name = "N")]
    pub limit: Option<u64>,
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl JmapContactCardQueryCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: JmapClient) -> Result<()> {
        let position = match &self.page_token.token {
            Some(token) => Some(
                token
                    .parse::<u64>()
                    .map_err(|_| page_token_invalid(token))?,
            ),
            None => None,
        };

        let filter = JmapContactCardFilter {
            in_address_book: self.in_address_book,
            text: self.text,
            ..Default::default()
        };
        let opts = JmapContactCardQueryOptions {
            filter: Some(filter),
            position,
            limit: self.limit,
            ..Default::default()
        };
        let out = client.contact_card_query(opts)?;

        let listed = (out.cards.len() + out.not_found.len()) as u64;
        let next = out.position + listed;
        let next_page = query_page_follows(out.total, listed, next, self.limit);

        printer.out(Paginated {
            inner: JmapContactCardsOutput {
                cards: out
                    .cards
                    .iter()
                    .map(serde_json::to_value)
                    .collect::<Result<_, _>>()?,
                state: None,
                query_state: Some(out.query_state),
            },
            next_page: next_page.then(|| next.to_string()),
        })
    }
}

/// Whether a page follows the one a query answered with `listed` ids,
/// the next one starting at `next`: below the query's `total`, else while
/// the page came back as full as `limit` asked; never after an empty page.
fn query_page_follows(total: Option<u64>, listed: u64, next: u64, limit: Option<u64>) -> bool {
    if listed == 0 {
        return false;
    }

    match total {
        Some(total) => next < total,
        None => limit.is_some_and(|limit| listed >= limit),
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use io_jmap::{client::JmapClientStd, rfc8620::session::JmapSession};
    use pimalaya_cli::printer::CapturePrinter;
    use secrecy::SecretString;
    use serde_json::{Value, json};

    use super::{JmapContactCardQueryCommand, query_page_follows};
    use crate::{
        jmap::client::JmapClient,
        script::{self, Requests, Script},
    };

    /// The client over a JMAP server answering a query at `position` with
    /// the cards `ids` out of `total`, and the requests it receives.
    fn answering(position: u64, ids: &[&str], total: u64) -> (JmapClient, Requests) {
        let session: JmapSession = serde_json::from_value(json!({
            "username": "", "accounts": {}, "capabilities": {},
            "primaryAccounts": {"urn:ietf:params:jmap:contacts": "a1"},
            "apiUrl": "https://jmap.example.org/api/", "downloadUrl": "",
            "uploadUrl": "", "eventSourceUrl": "", "state": "",
        }))
        .unwrap();
        let cards: Vec<Value> = ids
            .iter()
            .map(|id| json!({"id": id, "addressBookIds": {"book": true}, "@type": "Card"}))
            .collect();
        let body = json!({"methodResponses": [
            ["ContactCard/query", {
                "accountId": "a1", "queryState": "q1", "canCalculateChanges": false,
                "position": position, "ids": ids, "total": total,
            }, "c0"],
            ["ContactCard/get", {
                "accountId": "a1", "state": "s1", "notFound": [], "list": cards,
            }, "c1"],
        ], "sessionState": "s"});
        let response = script::response(200, "application/json", &body.to_string());
        let (script, requests) = Script::new([response]);
        let auth = SecretString::from("Bearer token");
        let inner = JmapClientStd::from_parts(script, auth, session);

        (JmapClient::new(inner), requests)
    }

    /// Runs `query` with `args` against `client`, returning its JSON.
    fn query(args: &[&str], client: JmapClient) -> Value {
        let argv = ["query"].iter().chain(args);
        let cmd = JmapContactCardQueryCommand::try_parse_from(argv).unwrap();
        let mut printer = CapturePrinter::default();
        cmd.execute(&mut printer, client).unwrap();
        printer.into_single().unwrap()
    }

    /// A query `--limit` stops prints the position the next page starts
    /// at as `nextPage`, which `--page-token` sends back as the position;
    /// the last page prints `null`.
    #[test]
    fn a_limited_query_names_its_next_page() {
        let (client, _) = answering(0, &["c1", "c2"], 3);
        let json = query(&["--limit", "2"], client);
        assert_eq!(json["nextPage"], "2", "{json}");
        assert_eq!(json["list"].as_array().unwrap().len(), 2, "{json}");

        let (client, requests) = answering(2, &["c3"], 3);
        let json = query(&["--limit", "2", "--page-token", "2"], client);
        assert_eq!(json["nextPage"], Value::Null, "{json}");

        let request = String::from_utf8_lossy(&requests.lock().unwrap()[0]).into_owned();
        assert!(request.contains(r#""position":2"#), "{request}");
    }

    /// A query prints its `queryState` under that key, never under the
    /// `state` a get prints and `contact-card changes` takes.
    #[test]
    fn a_query_prints_its_query_state() {
        let (client, _) = answering(0, &["c1"], 1);
        let json = query(&[], client);
        assert_eq!(json["queryState"], "q1", "{json}");
        assert!(json.get("state").is_none(), "{json}");
    }

    /// A page follows below the total, or, with none, after a full page;
    /// never after the last one nor after an empty one.
    #[test]
    fn a_page_follows_until_the_total() {
        assert!(query_page_follows(Some(5), 2, 2, Some(2)));
        assert!(!query_page_follows(Some(4), 2, 4, Some(2)));
        assert!(query_page_follows(None, 2, 2, Some(2)));
        assert!(!query_page_follows(None, 1, 1, Some(2)));
        assert!(!query_page_follows(None, 3, 3, None));
        assert!(!query_page_follows(Some(5), 0, 2, Some(2)));
    }
}
