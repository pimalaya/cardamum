use std::{
    fmt, fs,
    io::{Read, Write, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use comfy_table::{Cell, Row, Table};
use io_webdav::{
    coroutine::{WebdavCoroutine, WebdavCoroutineState, WebdavYield},
    rfc4918::{GETETAG, WebdavMultistatus, report::WebdavReport},
    rfc6352::addressbook::ADDRESS_DATA,
};
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::{carddav::client::CarddavClient, shared::table::style_from_preset};

/// Run a REPORT with an arbitrary XML body against an addressbook.
///
/// The escape hatch for custom `addressbook-query` / `-multiget` /
/// `sync-collection` (or any) REPORTs the typed subcommands don't cover.
/// The XML body comes from a file, inline, or `-` for stdin; the parsed
/// multistatus is printed.
///
/// JSON output: `{"responses": [{"href", "status", "etag",
/// "data_bytes"}], "sync_token"}`.
#[derive(Debug, Parser)]
pub struct CarddavReportRawCommand {
    /// Identifier of the addressbook to run the REPORT against.
    #[arg(value_name = "ADDRESSBOOK")]
    pub addressbook_id: String,
    /// XML REPORT body: a file path, raw XML, or `-` for stdin.
    #[arg(value_name = "XML")]
    pub xml: String,
    /// `Depth` header (0 or 1).
    #[arg(long, default_value_t = 1)]
    pub depth: u8,
}

impl CarddavReportRawCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CarddavClient) -> Result<()> {
        let preset = client.account.table_preset().to_string();
        let body = read_xml(&self.xml)?;

        let home = client
            .addressbook_home_set
            .clone()
            .ok_or_else(|| anyhow!("addressbook home-set is not resolved"))?;
        let path = format!(
            "{}/{}",
            home.path().trim_end_matches('/'),
            self.addressbook_id.trim_matches('/')
        );
        let base_url = client.base_url.clone();
        let user_agent = client.user_agent.clone();
        let auth = client.auth().clone();

        let coroutine = WebdavReport::new(&base_url, &auth, &user_agent, &path, self.depth, body);
        let multistatus = run_report(&mut client, coroutine)?;

        printer.out(RawReport {
            preset,
            sync_token: multistatus.sync_token,
            responses: multistatus.responses.iter().map(EntryRow::from).collect(),
        })
    }
}

/// Reads the XML body: `-` reads stdin, an existing file is read,
/// otherwise a value starting with `<` is treated as inline XML.
fn read_xml(source: &str) -> Result<Vec<u8>> {
    if source == "-" {
        let mut buf = Vec::new();
        stdin()
            .read_to_end(&mut buf)
            .context("Read REPORT body from stdin error")?;
        return Ok(buf);
    }

    let path = PathBuf::from(source);
    if path.is_file() {
        return fs::read(&path)
            .with_context(|| format!("Read REPORT body from `{}` error", path.display()));
    }

    if source.trim_start().starts_with('<') {
        return Ok(source.as_bytes().to_vec());
    }

    bail!("Source `{source}` is neither a readable file nor XML");
}

/// Pumps a [`WebdavReport`] coroutine against the client's connected stream.
fn run_report(
    client: &mut CarddavClient,
    mut coroutine: WebdavReport,
) -> Result<WebdavMultistatus> {
    let mut buf = [0u8; 8 * 1024];
    let mut arg: Option<&[u8]> = None;

    loop {
        match coroutine.resume(arg.take()) {
            WebdavCoroutineState::Complete(result) => return Ok(result?),
            WebdavCoroutineState::Yielded(WebdavYield::WantsWrite(bytes)) => {
                client.stream.write_all(&bytes)?;
            }
            WebdavCoroutineState::Yielded(WebdavYield::WantsRead) => {
                let n = client.stream.read(&mut buf)?;
                arg = Some(&buf[..n]);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RawReport {
    #[serde(skip)]
    pub preset: String,
    pub responses: Vec<EntryRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_token: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EntryRow {
    pub href: String,
    pub status: Option<u16>,
    pub etag: Option<String>,
    pub data_bytes: usize,
}

impl From<&io_webdav::rfc4918::WebdavResponseEntry> for EntryRow {
    fn from(entry: &io_webdav::rfc4918::WebdavResponseEntry) -> Self {
        Self {
            href: entry.href.clone(),
            status: entry.status,
            etag: entry
                .text(GETETAG)
                .map(|raw| raw.trim_matches('"').to_string()),
            data_bytes: entry.text(ADDRESS_DATA).map(str::len).unwrap_or(0),
        }
    }
}

impl fmt::Display for RawReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_style(style_from_preset(&self.preset))
            .set_header(Row::from([
                Cell::new("HREF"),
                Cell::new("STATUS"),
                Cell::new("ETAG"),
                Cell::new("DATA-BYTES"),
            ]))
            .add_rows(self.responses.iter().map(|entry| {
                let status = entry.status.map(|s| s.to_string()).unwrap_or_default();
                let mut row = Row::new();
                row.max_height(1)
                    .add_cell(Cell::new(&entry.href))
                    .add_cell(Cell::new(status))
                    .add_cell(Cell::new(entry.etag.as_deref().unwrap_or("")))
                    .add_cell(Cell::new(entry.data_bytes));
                row
            }));

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        if let Some(token) = &self.sync_token {
            writeln!(f, "sync-token: {token}")?;
        }
        Ok(())
    }
}
