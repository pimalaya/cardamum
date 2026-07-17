//! Google People API wizard (Google accounts).
//!
//! The People API is bearer-token-only: the wizard collects the token
//! secret, typically an Ortie shell command since tokens expire and
//! need refreshing. It does not connect; the wizard validates the whole
//! account once at the end (see [`crate::account::check`]).

use anyhow::Result;

use crate::{
    config::{GoogleAuthConfig, GoogleConfig},
    wizard::secret,
};

/// Runs the Google People wizard, returning a ready [`GoogleConfig`].
pub fn configure() -> Result<GoogleConfig> {
    // The note is guidance, not config, so it goes to stderr: only the
    // generated TOML must land on stdout for `cardamum > <config>`.
    eprintln!(
        "Google People uses OAuth 2.0 tokens; issue and refresh them with an external manager such as Ortie"
    );

    let token = secret::configure("Google People access token", Some("ortie token show"))?;

    Ok(GoogleConfig {
        tls: Default::default(),
        alpn: vec!["http/1.1".to_string()],
        auth: GoogleAuthConfig { token },
    })
}
