//! JMAP wizard.
//!
//! A discovery entry pins the session endpoint and the authentication
//! method, so [`configure_discovered`] prompts only the credentials. It
//! does not connect; the wizard validates the whole account once at the
//! end (see [`crate::account::check`]).

use anyhow::{Result, bail};
use pimalaya_cli::prompt;

use crate::{
    config::{JmapAuthConfig, JmapConfig},
    wizard::{
        search::{Discovered, DiscoveredAuth, DiscoveredKind},
        secret,
    },
};

/// Configures JMAP from a discovered entry: the endpoint and the
/// authentication method are pinned, only the credentials are prompted.
pub fn configure_discovered(email: &str, discovered: &Discovered) -> Result<JmapConfig> {
    let DiscoveredKind::Jmap(server) = &discovered.kind else {
        bail!("Expected a JMAP configuration");
    };

    let auth = match discovered.auth {
        DiscoveredAuth::Password => {
            let default_login = discovered.login_default(email);
            let username = prompt::text("JMAP username:", default_login.as_deref())?;
            let password = secret::configure("JMAP password", None)?;
            JmapAuthConfig::Basic { username, password }
        }
        DiscoveredAuth::Token => {
            let token = secret::configure("JMAP API token", Some("ortie token show"))?;
            JmapAuthConfig::Bearer { token }
        }
    };

    Ok(JmapConfig {
        server: server.to_string(),
        tls: Default::default(),
        alpn: io_jmap::client::JmapClientStd::default_alpn(),
        auth,
    })
}
