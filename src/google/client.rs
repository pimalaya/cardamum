//! Cardamum wrapper around [`io_people::v1::client::PeopleClientStd`]
//! that bundles the merged [`Account`] alongside the connected client.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_people::v1::client::{PeopleClientStd, PeopleClientStdConnectOptions};
use secrecy::ExposeSecret;

use crate::{
    account::context::Account,
    config::{AccountConfig, Config},
};

pub struct GoogleClient {
    inner: PeopleClientStd,
    pub account: Account,
}

impl Deref for GoogleClient {
    type Target = PeopleClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for GoogleClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Builds the merged [`Account`] from the already-resolved config and
/// account, then opens the People client. Bails when the account has no
/// `[google]` block.
pub fn build_google_client(
    config: Config,
    name: String,
    mut account_config: AccountConfig,
) -> Result<GoogleClient> {
    let google_config = account_config
        .google
        .take()
        .ok_or_else(|| anyhow!("Google People config is missing for account `{name}`"))?;

    let token = google_config.auth.token.get()?;
    let options = PeopleClientStdConnectOptions {
        tls: google_config.tls.into_tls(google_config.alpn),
    };
    let inner = PeopleClientStd::connect(token.expose_secret(), options)?;

    let account = Account::from(config).merge(Account::from(account_config));
    Ok(GoogleClient { inner, account })
}
