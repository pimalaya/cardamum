//! # Raw request command
//!
//! Sends an arbitrary Google People API request and prints the JSON
//! response.

use anyhow::{Context, Result, bail};
use clap::Parser;
use io_gpeople::v1::send::{GPEOPLE_API_BASE, GpeopleSend, GpeopleSendOutput};
use pimalaya_cli::{error::request_origin, printer::Printer};
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

/// The request URL: a full URL (one holding `://`) as-is, else a path
/// joined onto the People base, refused off the People origin. The path
/// joins as `./<path>`, so the colon of a custom method on a collection
/// (`people:createContact`) is not read as a scheme.
fn resolve_url(path: &str) -> Result<Url> {
    let base = Url::parse(GPEOPLE_API_BASE)?;
    let url = match path.contains("://") {
        true => Url::parse(path)?,
        false => base.join(&format!("./{}", path.trim_start_matches('/')))?,
    };

    if url.origin() != base.origin() {
        return Err(request_origin(
            &url.origin().ascii_serialization(),
            &base.origin().ascii_serialization(),
        ));
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use pimalaya_cli::error::{CommonCode, code_of};

    use super::resolve_url;

    /// A path and a full URL on the People origin resolve.
    #[test]
    fn a_url_on_the_people_origin_resolves() {
        let url = resolve_url("people/me").unwrap();
        assert_eq!(url.as_str(), "https://people.googleapis.com/v1/people/me");

        let full = "https://people.googleapis.com/v1/contactGroups";
        assert_eq!(resolve_url(full).unwrap().as_str(), full);
    }

    /// A path naming a custom method of a collection (`people:createContact`)
    /// joins onto the People base, its colon not read as a scheme.
    #[test]
    fn a_custom_method_path_resolves_on_the_people_origin() {
        for (path, url) in [
            (
                "people:createContact",
                "https://people.googleapis.com/v1/people:createContact",
            ),
            (
                "/contactGroups:batchGet?resourceNames=contactGroups/a",
                "https://people.googleapis.com/v1/contactGroups:batchGet?resourceNames=contactGroups/a",
            ),
            (
                "people/c1:updateContact",
                "https://people.googleapis.com/v1/people/c1:updateContact",
            ),
        ] {
            assert_eq!(resolve_url(path).expect(path).as_str(), url);
        }
    }

    /// A URL off the People origin is refused before the token is sent,
    /// coded `request-origin`: another host, plain HTTP, another port, and
    /// an absolute URL a path would join over the base.
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
            let err = resolve_url(url).expect_err(url);
            assert_eq!(code_of::<CommonCode>(&err), Some(CommonCode::RequestOrigin));
            assert!(
                err.to_string().contains("off the API origin"),
                "{url}: {err}"
            );
        }
    }
}
