//! # Raw request command
//!
//! Sends an arbitrary Google People API request and prints the JSON
//! response.

use anyhow::{Context, Result, bail};
use clap::Parser;
use io_gpeople::v1::send::{GPEOPLE_API_BASE, GpeopleSend, GpeopleSendOutput};
use pimalaya_cli::printer::Printer;
use serde_json::Value;
use url::Url;

use crate::{gpeople::client::GpeopleClient, shared::raw_json::RawJsonOutput};

/// Send a raw Google People API request and print the JSON response.
///
/// The path is joined onto the People `v1/` base, or a full URL is used
/// as-is. JSON output: the raw People response.
#[derive(Debug, Parser)]
pub struct GpeopleRequestCommand {
    /// HTTP method: get, post, put, patch, delete.
    #[arg(value_name = "METHOD")]
    pub method: String,
    /// People path (joined onto `v1/`) or a full URL.
    #[arg(value_name = "PATH")]
    pub path: String,
    /// Raw JSON body for post / put / patch.
    #[arg(value_name = "JSON")]
    pub body: Option<String>,
}

impl GpeopleRequestCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GpeopleClient) -> Result<()> {
        let url = resolve_url(&self.path)?;
        let auth = client.auth.clone();

        let send: GpeopleSend<Value> = match self.method.to_ascii_uppercase().as_str() {
            "GET" => GpeopleSend::get(&auth, url),
            "DELETE" => GpeopleSend::delete(&auth, url),
            "POST" => GpeopleSend::post_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            "PUT" => GpeopleSend::put_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            "PATCH" => GpeopleSend::patch_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            other => bail!("Unsupported method `{other}` (use get/post/put/patch/delete)"),
        };
        let out: GpeopleSendOutput<Value> = client.run(send)?;

        printer.out(RawJsonOutput(out.response))
    }
}

/// Parses the optional raw body, an absent one meaning JSON null.
fn parse_body(body: Option<&str>) -> Result<Value> {
    serde_json::from_str(body.unwrap_or("null")).context("Parse request body JSON error")
}

/// The request URL: a full URL as-is, else joined onto the People base.
fn resolve_url(path: &str) -> Result<Url> {
    if path.starts_with("http") {
        Ok(Url::parse(path)?)
    } else {
        Ok(Url::parse(GPEOPLE_API_BASE)?.join(path.trim_start_matches('/'))?)
    }
}
