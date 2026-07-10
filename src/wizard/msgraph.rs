//! Microsoft Graph wizard.
//!
//! The Graph API is bearer-token-only: the wizard collects the token
//! secret (typically an ortie shell command, since tokens expire and
//! need refreshing) and the user id, then tests the connection by
//! listing the contact folders; a failure re-runs the prompts.

use anyhow::Result;
use pimalaya_cli::{prompt, spinner::Spinner};

use crate::{
    config::{MsgraphAuthConfig, MsgraphConfig},
    msgraph::backend::MsgraphBackend,
    wizard::secret,
};

/// Runs the Microsoft Graph wizard until the connection succeeds;
/// `existing` seeds the defaults when editing.
pub fn configure(existing: Option<&MsgraphConfig>) -> Result<MsgraphConfig> {
    let tls_config = existing.map(|c| c.tls.clone()).unwrap_or_default();

    loop {
        let default_user_id = existing.map(|c| c.user_id.as_str()).unwrap_or("me");
        let user_id = prompt::text("Graph user id:", Some(default_user_id))?;
        let token = secret::configure("Graph access token", Some("ortie token show"))?;

        let config = MsgraphConfig {
            user_id,
            tls: tls_config.clone(),
            alpn: vec!["http/1.1".into()],
            auth: MsgraphAuthConfig { token },
        };

        let spinner = Spinner::start("Testing connection");

        let test =
            MsgraphBackend::new(config.clone()).and_then(|mut client| client.list_addressbooks());

        match test {
            Ok(_) => {
                spinner.success("Connection successful");
                return Ok(config);
            }
            Err(err) => spinner.failure(format!("Connection failed: {err}")),
        }
    }
}
