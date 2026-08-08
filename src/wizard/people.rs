//! Google People API wizard (Google accounts).
//!
//! The People API is bearer-token-only: the wizard collects the token
//! secret, typically read from an external broker such as Ortie since
//! tokens expire and need refreshing. It does not connect; the wizard
//! validates the whole account once at the end (see
//! [`crate::account::check`]).

use anyhow::Result;

use crate::{
    config::{PeopleAuthConfig, PeopleConfig},
    wizard::secret,
};

/// Runs the Google People wizard, returning a ready [`PeopleConfig`].
pub fn configure(account_name: &str) -> Result<PeopleConfig> {
    eprintln!(
        "Google People uses OAuth 2.0 tokens; issue and refresh them with an external broker such as Ortie."
    );

    let token = secret::configure_token(
        "Google People access token",
        &format!("{account_name}-people"),
        true,
    )?;

    Ok(PeopleConfig {
        tls: Default::default(),
        alpn: vec!["http/1.1".to_string()],
        auth: PeopleAuthConfig { token },
    })
}
