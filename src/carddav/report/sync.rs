//! # Sync report command
//!
//! Runs the RFC 6578 `sync-collection` REPORT and renders the changed
//! and vanished hrefs.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use io_webdav::rfc6578::sync_collection::{
    WebdavSyncChange, WebdavSyncCollectionOptions, WebdavSyncDelta,
};
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::carddav::client::CarddavClient;

/// Report the changes since a sync token (RFC 6578).
///
/// Omit `--sync-token` for an initial sync, then feed the returned
/// token back to the next call. Only ETags are fetched, so pair with
/// `report multiget` to pull the changed cards.
///
/// JSON output: `{"changed": [{"href", "etag"}], "vanished": [...],
/// "syncToken", "truncated"}`.
#[derive(Debug, Parser)]
pub struct CarddavReportSyncCommand {
    /// Identifier of the addressbook to sync.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// Sync token from a previous sync; omit for an initial sync.
    #[arg(long, value_name = "TOKEN")]
    pub sync_token: Option<String>,
    /// Enumerate with a Depth 1 PROPFIND instead of the REPORT.
    ///
    /// For a server implementing no `sync-collection`: it lists every
    /// member and returns no token, so the result is a full snapshot to
    /// diff.
    #[arg(long)]
    pub fallback: bool,
}

impl CarddavReportSyncCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let opts = WebdavSyncCollectionOptions {
            fallback: self.fallback,
        };
        let delta = client.sync_cards(&self.addressbook_id, self.sync_token.as_deref(), opts)?;

        printer.out(CarddavReportSyncOutput::new(delta))
    }
}

/// Changes a `sync-collection` REPORT reported for one addressbook.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CarddavReportSyncOutput {
    /// The resources created or updated since the given token.
    pub changed: Vec<ChangeRow>,
    /// The hrefs of the resources that vanished since it.
    pub vanished: Vec<String>,
    /// The token to feed to the next sync, when the server sent one.
    pub sync_token: Option<String>,
    /// Whether the server cut the listing short (RFC 6578 section 3.6).
    pub truncated: bool,
}

/// One created or updated resource: its href and ETag.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRow {
    /// The changed resource's href.
    pub href: String,
    /// Its ETag, when the server sent one.
    pub etag: Option<String>,
}

impl From<WebdavSyncChange> for ChangeRow {
    fn from(change: WebdavSyncChange) -> Self {
        Self {
            href: change.href,
            etag: change.etag,
        }
    }
}

impl CarddavReportSyncOutput {
    fn new(delta: WebdavSyncDelta) -> Self {
        Self {
            changed: delta.changed.into_iter().map(ChangeRow::from).collect(),
            vanished: delta.vanished,
            sync_token: delta.sync_token,
            truncated: delta.truncated,
        }
    }
}

impl fmt::Display for CarddavReportSyncOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.changed.is_empty() || !self.vanished.is_empty() {
            let mut columns = Columns::new(["STATUS", "ETAG", "HREF"]);

            for change in &self.changed {
                let etag = change.etag.as_deref().map(sanitize::text);

                columns.push([
                    String::from("changed"),
                    etag.map(|etag| etag.into_owned()).unwrap_or_default(),
                    sanitize::text(&change.href).into_owned(),
                ]);
            }

            for href in &self.vanished {
                columns.push([
                    String::from("vanished"),
                    String::new(),
                    sanitize::text(href).into_owned(),
                ]);
            }

            writeln!(f, "{columns}")?;
        }

        let token = self.sync_token.as_deref().map(sanitize::text);
        let token = token.unwrap_or("(none)".into());
        writeln!(f, "sync-token: {token}")?;
        if self.truncated {
            writeln!(
                f,
                "truncated: run sync again from this token to drain the rest"
            )?;
        }
        Ok(())
    }
}
