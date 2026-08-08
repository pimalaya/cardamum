//! JMAP wizard.
//!
//! A discovery entry pins the session endpoint, the HTTP authentication
//! scheme is picked among the advertised ones (skipped when only one
//! qualifies), then its credentials are prompted. It does not connect;
//! the wizard validates the whole account once at the end (see
//! [`crate::account::check`]).

use anyhow::{Result, bail};
use pimalaya_cli::prompt;

use crate::{
    config::{JmapAuthConfig, JmapConfig},
    wizard::{
        search::{AuthCaps, Discovered, DiscoveredKind},
        secret,
    },
};

const BASIC: &str = "Basic (username + password)";
const BEARER: &str = "Bearer (API token)";

/// Configures JMAP from a discovered entry: the endpoint is pinned, the
/// authentication scheme is picked among those advertised, and only its
/// credentials are prompted.
pub fn configure_discovered(
    account_name: &str,
    email: &str,
    discovered: &Discovered,
) -> Result<JmapConfig> {
    let DiscoveredKind::Jmap(server) = &discovered.kind else {
        bail!("Expected a JMAP configuration");
    };

    let auth = prompt_auth(
        account_name,
        discovered.login_default(email).as_deref(),
        discovered.auth,
    )?;

    Ok(JmapConfig {
        server: server.to_string(),
        tls: Default::default(),
        alpn: io_jmap::client::JmapClientStd::default_alpn(),
        auth,
    })
}

/// Prompts the HTTP authentication scheme from `caps` (both offered when
/// none was advertised), then its credentials. The Bearer token flow shows
/// the OAuth brokers only when a grant was advertised.
fn prompt_auth(
    account_name: &str,
    login_hint: Option<&str>,
    caps: AuthCaps,
) -> Result<JmapAuthConfig> {
    let mut schemes = Vec::new();
    if caps.basic || !caps.any() {
        schemes.push(BASIC);
    }
    if caps.token() || !caps.any() {
        schemes.push(BEARER);
    }

    let scheme = if schemes.len() == 1 {
        schemes[0]
    } else {
        prompt::item("JMAP authentication:", schemes, None)?
    };

    let key = format!("{account_name}-jmap");
    Ok(match scheme {
        BASIC => {
            let username = prompt::text("Login:", login_hint)?;
            let password = secret::configure_password("JMAP password", &key)?;
            JmapAuthConfig::Basic { username, password }
        }
        BEARER => {
            let token = secret::configure_token("JMAP API token", &key, caps.oauth || !caps.any())?;
            JmapAuthConfig::Bearer { token }
        }
        _ => unreachable!(),
    })
}
