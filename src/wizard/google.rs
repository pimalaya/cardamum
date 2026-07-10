//! Google People wizard.
//!
//! The People API is bearer-token-only: the wizard collects the token
//! secret (typically an ortie shell command, since tokens expire and
//! need refreshing), then tests the connection by listing the contact
//! groups; a failure re-runs the prompts.

use anyhow::Result;
use pimalaya_cli::spinner::Spinner;

use crate::{
    config::{GoogleAuthConfig, GoogleConfig},
    google::backend::GoogleBackend,
    wizard::secret,
};

/// Runs the Google People wizard until the connection succeeds;
/// `existing` seeds the defaults when editing.
pub fn configure(existing: Option<&GoogleConfig>) -> Result<GoogleConfig> {
    let tls_config = existing.map(|c| c.tls.clone()).unwrap_or_default();

    loop {
        let token = secret::configure("People access token", Some("ortie token show"))?;

        let config = GoogleConfig {
            tls: tls_config.clone(),
            alpn: vec!["http/1.1".into()],
            auth: GoogleAuthConfig { token },
        };

        let spinner = Spinner::start("Testing connection");

        let test =
            GoogleBackend::new(config.clone()).and_then(|mut client| client.list_addressbooks());

        match test {
            Ok(_) => {
                spinner.success("Connection successful");
                return Ok(config);
            }
            Err(err) => spinner.failure(format!("Connection failed: {err}")),
        }
    }
}
