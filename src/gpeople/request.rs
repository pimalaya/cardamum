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
/// as-is. A URL off the People origin (another host, scheme or port) is
/// refused before anything is sent, the account's token going to People
/// alone. JSON output: `{response}`, the raw People response.
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

        printer.out(RawJsonOutput {
            response: out.response,
        })
    }
}

/// Parses the optional raw body, an absent one meaning JSON null.
fn parse_body(body: Option<&str>) -> Result<Value> {
    serde_json::from_str(body.unwrap_or("null")).context("Parse request body JSON error")
}

/// The request URL: a full URL as-is, else joined onto the People base,
/// refused off the People origin.
fn resolve_url(path: &str) -> Result<Url> {
    let base = Url::parse(GPEOPLE_API_BASE)?;
    let url = match path.starts_with("http") {
        true => Url::parse(path)?,
        false => base.join(path.trim_start_matches('/'))?,
    };

    if url.origin() != base.origin() {
        bail!(
            "Cannot send a request to `{}`: it is off the API origin `{}`, \
             which alone receives the account's token",
            url.origin().ascii_serialization(),
            base.origin().ascii_serialization(),
        );
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::resolve_url;

    /// A path and a full URL on the People origin resolve.
    #[test]
    fn a_url_on_the_people_origin_resolves() {
        let url = resolve_url("people/me").unwrap();
        assert_eq!(url.as_str(), "https://people.googleapis.com/v1/people/me");

        let full = "https://people.googleapis.com/v1/contactGroups";
        assert_eq!(resolve_url(full).unwrap().as_str(), full);
    }

    /// A URL off the People origin is refused before the token is sent:
    /// another host, plain HTTP, another port, and an absolute URL a path
    /// would join over the base.
    #[test]
    fn a_url_off_the_people_origin_is_refused() {
        for url in [
            "https://attacker.example/v1/people/me",
            "http://people.googleapis.com/v1/people/me",
            "https://people.googleapis.com:8443/v1/people/me",
            "https://www.googleapis.com/drive/v3/files",
            "HTTPS://attacker.example/people/me",
            "ftp://attacker.example/people/me",
        ] {
            let err = resolve_url(url).expect_err(url).to_string();
            assert!(err.contains("off the API origin"), "{url}: {err}");
        }
    }
}
