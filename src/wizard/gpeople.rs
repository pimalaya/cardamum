//! # Google People wizard
//!
//! The People API is bearer-token-only, so the wizard collects the token
//! secret, typically read from an external broker such as Ortie since
//! tokens expire and need refreshing.
//!
//! It does not connect: the wizard validates the whole account once at
//! the end (see [`crate::account::check`]).

use anyhow::Result;

use crate::{
    config::{GpeopleAuthConfig, GpeopleConfig},
    wizard::secret,
};

/// Runs the Google People wizard, returning a ready [`GpeopleConfig`].
pub fn configure(account_name: &str) -> Result<GpeopleConfig> {
    eprintln!(
        "Google People uses OAuth 2.0 tokens; issue and refresh them with an external broker such as Ortie."
    );

    let token = secret::configure_token(
        "Google People access token",
        &format!("{account_name}-gpeople"),
        true,
    )?;

    Ok(GpeopleConfig {
        tls: Default::default(),
        proxy: None,
        alpn: vec!["http/1.1".to_string()],
        auth: GpeopleAuthConfig { token },
    })
}
