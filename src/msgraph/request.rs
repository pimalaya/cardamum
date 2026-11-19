//! # Raw request command
//!
//! Sends an arbitrary Microsoft Graph request and prints the JSON response,
//! the escape hatch for the endpoints no command covers.

use anyhow::{Context, Result, bail};
use clap::Parser;
use io_msgraph::v1::send::{MSGRAPH_API_BASE, MsgraphSend, MsgraphSendOutput};
use pimalaya_cli::{error::request_origin, printer::Printer};
use serde_json::Value;
use url::Url;

use crate::{msgraph::client::MsgraphClient, shared::raw_json::RawJsonOutput};

/// Send a raw Microsoft Graph request and print the JSON response.
///
/// The path is joined onto the Graph `v1.0/` base, or a full URL is used
/// as-is. A URL off the Graph origin (another host, scheme or port) is
/// refused before anything is sent, the account's token going to Graph
/// alone. JSON output: `{response}`, the raw Graph response.
#[derive(Debug, Parser)]
pub struct MsgraphRequestCommand {
    /// HTTP method: get, post, patch, delete.
    #[arg(value_name = "METHOD")]
    pub method: String,
    /// Graph path (joined onto `v1.0/`) or a full URL.
    #[arg(value_name = "PATH")]
    pub path: String,
    /// Raw JSON body for post / patch.
    #[arg(value_name = "JSON")]
    pub body: Option<String>,
}

impl MsgraphRequestCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let url = resolve_url(&self.path)?;
        let auth = client.auth.clone();

        let send: MsgraphSend<Value> = match self.method.to_ascii_uppercase().as_str() {
            "GET" => MsgraphSend::get(&auth, url),
            "DELETE" => MsgraphSend::delete(&auth, url),
            "POST" => MsgraphSend::post_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            "PATCH" => MsgraphSend::patch_json(&auth, url, &parse_body(self.body.as_deref())?)?,
            other => bail!("Unsupported method `{other}` (use get/post/patch/delete)"),
        };
        let out: MsgraphSendOutput<Value> = client.run(send)?;

        printer.out(RawJsonOutput {
            response: out.response,
        })
    }
}

/// Parses the raw body argument, an absent body meaning JSON `null`.
fn parse_body(body: Option<&str>) -> Result<Value> {
    serde_json::from_str(body.unwrap_or("null")).context("Parse request body JSON error")
}

/// Resolves a Graph path onto the `v1.0/` base, a full URL being taken
/// as-is, and refuses a URL off the Graph origin.
fn resolve_url(path: &str) -> Result<Url> {
    let base = Url::parse(MSGRAPH_API_BASE)?;
    let url = match path.starts_with("http") {
        true => Url::parse(path)?,
        false => base.join(path.trim_start_matches('/'))?,
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

    /// A path and a full URL on the Graph origin resolve, the `beta`
    /// endpoint and a next link included.
    #[test]
    fn a_url_on_the_graph_origin_resolves() {
        let url = resolve_url("/me/contacts").unwrap();
        assert_eq!(url.as_str(), "https://graph.microsoft.com/v1.0/me/contacts");

        let next = "https://graph.microsoft.com/v1.0/me/contacts?$skiptoken=x";
        assert_eq!(resolve_url(next).unwrap().as_str(), next);

        let beta = "https://graph.microsoft.com/beta/me";
        assert_eq!(resolve_url(beta).unwrap().as_str(), beta);
    }

    /// A URL off the Graph origin is refused before the token is sent,
    /// coded `request-origin`: another host, plain HTTP, another port, and
    /// an absolute URL a path would join over the base.
    #[test]
    fn a_url_off_the_graph_origin_is_refused() {
        for url in [
            "https://attacker.example/v1.0/me",
            "http://graph.microsoft.com/v1.0/me",
            "https://graph.microsoft.com:8443/v1.0/me",
            "https://graph.microsoft.com.attacker.example/v1.0/me",
            "HTTPS://attacker.example/me",
            "ftp://attacker.example/me",
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
