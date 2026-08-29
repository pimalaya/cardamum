//! # Sync report command
//!
//! Runs the RFC 6578 `sync-collection` REPORT and renders the changed
//! and vanished hrefs.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, Row, Table};
use io_webdav::rfc6578::sync_collection::{
    WebdavSyncChange, WebdavSyncCollectionOptions, WebdavSyncDelta,
};
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::{carddav::client::CarddavClient, shared::table::style_from_preset};

/// Report the changes since a sync token (RFC 6578).
///
/// Omit `--sync-token` for an initial sync, then feed the returned
/// token back to the next call. Only ETags are fetched, so pair with
/// `report multiget` to pull the changed cards.
///
/// JSON output: `{"changed": [{"href", "etag"}], "vanished": [...],
/// "sync_token", "truncated"}`.
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
        let preset = client.account.table_preset().to_string();
        let opts = WebdavSyncCollectionOptions {
            fallback: self.fallback,
        };
        let delta = client.sync_cards(&self.addressbook_id, self.sync_token.as_deref(), opts)?;

        printer.out(SyncReport::new(preset, delta))
    }
}

/// Changes a `sync-collection` REPORT reported for one addressbook.
#[derive(Clone, Debug, Serialize)]
pub struct SyncReport {
    #[serde(skip)]
    pub preset: String,
    pub changed: Vec<ChangeRow>,
    pub vanished: Vec<String>,
    pub sync_token: Option<String>,
    pub truncated: bool,
}

/// One created or updated resource: its href and ETag.
#[derive(Clone, Debug, Serialize)]
pub struct ChangeRow {
    pub href: String,
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

impl SyncReport {
    fn new(preset: String, delta: WebdavSyncDelta) -> Self {
        Self {
            preset,
            changed: delta.changed.into_iter().map(ChangeRow::from).collect(),
            vanished: delta.vanished,
            sync_token: delta.sync_token,
            truncated: delta.truncated,
        }
    }
}

impl fmt::Display for SyncReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("STATUS"),
                Cell::new("HREF"),
                Cell::new("ETAG"),
            ]));
        for change in &self.changed {
            table.add_row(Row::from([
                Cell::new("changed"),
                Cell::new(&change.href),
                Cell::new(change.etag.as_deref().unwrap_or("")),
            ]));
        }
        for href in &self.vanished {
            table.add_row(Row::from([
                Cell::new("vanished"),
                Cell::new(href),
                Cell::new(""),
            ]));
        }

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        writeln!(
            f,
            "sync-token: {}",
            self.sync_token.as_deref().unwrap_or("(none)")
        )?;
        if self.truncated {
            writeln!(
                f,
                "truncated: run sync again from this token to drain the rest"
            )?;
        }
        Ok(())
    }
}
