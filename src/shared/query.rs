//! # Query
//!
//! The pimdir SEARCH query language (§8) as every card search reads it,
//! parsed by io-pimdir once, before any backend is reached, so a query is
//! accepted or refused the same way on every backend.

use anyhow::Result;
use ariadne::{Config, Label, Report, ReportKind, Source};
use io_pimdir::search::query::{PimdirQuery, PimdirQueryError, PimdirQueryErrorCode};
use pimalaya_cli::error::CodedError;

use crate::error::ErrorCode;

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

/// A refusal drawn over the query with ariadne, without colour like every
/// text output.
fn refusal_report(query: &str, err: &PimdirQueryError) -> String {
    let name = "query";
    let config = Config::default().with_color(false);
    // NOTE: an empty span past the end is drawn on the last character.
    let end = err.end.max(err.start + 1);
    let mut report = Report::build(ReportKind::Error, (name, err.start..end))
        .with_config(config)
        .with_code(err.code.as_str())
        .with_message("Query refused")
        .with_label(Label::new((name, err.start..end)).with_message(&err.message));

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
