//! Microsoft Graph API wizard (Microsoft accounts).
//!
//! The Graph API is bearer-token-only: the wizard collects the user id
//! and the token secret, typically an Ortie shell command since tokens
//! expire and need refreshing. It does not connect; the wizard
//! validates the whole account once at the end (see
//! [`crate::account::check`]).

use anyhow::Result;
use pimalaya_cli::prompt;

use crate::{
    config::{MsgraphAuthConfig, MsgraphConfig},
    wizard::secret,
};

/// Runs the Microsoft Graph wizard, returning a ready [`MsgraphConfig`].
pub fn configure() -> Result<MsgraphConfig> {
    // The note is guidance, not config, so it goes to stderr: only the
    // generated TOML must land on stdout for `cardamum > <config>`.
    eprintln!(
        "Microsoft Graph uses OAuth 2.0 tokens; issue and refresh them with an external manager such as Ortie"
    );

    let user_id = prompt::text("Microsoft Graph user id:", Some("me"))?;
    let token = secret::configure("Microsoft Graph access token", Some("ortie token show"))?;

    Ok(MsgraphConfig {
        user_id,
        tls: Default::default(),
        alpn: vec!["http/1.1".to_string()],
        auth: MsgraphAuthConfig { token },
    })
}
