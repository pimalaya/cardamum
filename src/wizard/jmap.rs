//! JMAP wizard.
//!
//! Collects the session endpoint and authentication, then tests the
//! connection by establishing the JMAP session (TLS,
//! `/.well-known/jmap` discovery); a failed connection re-runs the
//! prompts. A discovery entry pins the endpoint and the
//! authentication method, so only the secret is prompted.

use anyhow::Result;
use pimalaya_cli::{prompt, spinner::Spinner};

use crate::{
    config::{JmapAuthConfig, JmapConfig},
    jmap::backend::JmapBackend,
    wizard::{
        search::{Discovered, DiscoveredAuth, DiscoveredKind},
        secret,
    },
};

const BASIC: &str = "Basic (username + password)";
const BEARER: &str = "Bearer (API or OAuth 2.0 access token)";
const AUTHS: [&str; 2] = [BASIC, BEARER];

/// Runs the JMAP wizard until the session opens: the discovered entry
/// (when present) pins the server and the authentication method,
/// `existing` seeds the defaults when editing.
pub fn configure(
    email: &str,
    existing: Option<&JmapConfig>,
    discovered: Option<&Discovered>,
) -> Result<JmapConfig> {
    let tls_config = existing.map(|c| c.tls.clone()).unwrap_or_default();

    loop {
        let server = match discovered.map(|d| &d.kind) {
            Some(DiscoveredKind::Jmap(url)) => url.to_string(),
            _ => {
                let default_server = existing.map(|c| c.server.clone()).unwrap_or_else(|| {
                    email
                        .rsplit_once('@')
                        .map(|(_, d)| d)
                        .unwrap_or(email)
                        .to_string()
                });
                prompt::text(
                    "JMAP server (bare authority or full session URL):",
                    Some(default_server.as_str()),
                )?
            }
        };

        let strategy = match discovered.map(|d| d.auth) {
            Some(DiscoveredAuth::Password) => BASIC,
            Some(DiscoveredAuth::Bearer | DiscoveredAuth::Oauth) => BEARER,
            None => {
                let default = match existing.map(|c| &c.auth) {
                    Some(JmapAuthConfig::Basic { .. }) => Some(BASIC),
                    Some(_) => Some(BEARER),
                    None => None,
                };
                prompt::item("JMAP authentication strategy:", AUTHS, default)?
            }
        };

        let auth = match strategy {
            BASIC => {
                let default_username = discovered
                    .and_then(|d| d.username.clone())
                    .unwrap_or_else(|| email.to_string());
                let username = prompt::text("JMAP username:", Some(default_username.as_str()))?;
                let password = secret::configure("JMAP password", None)?;
                JmapAuthConfig::Basic { username, password }
            }
            BEARER => {
                let token = secret::configure("JMAP bearer token", Some("ortie token show"))?;
                JmapAuthConfig::Bearer { token }
            }
            _ => unreachable!(),
        };

        let config = JmapConfig {
            server,
            tls: tls_config.clone(),
            alpn: io_jmap::client::default_alpn(),
            auth,
        };

        let spinner = Spinner::start("Testing connection");

        match JmapBackend::new(config.clone()) {
            Ok(_) => {
                spinner.success("Connection successful");
                return Ok(config);
            }
            Err(err) => spinner.failure(format!("Connection failed: {err}")),
        }
    }
}
