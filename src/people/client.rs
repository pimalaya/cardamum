//! # People client
//!
//! Wraps [`io_people::v1::client::PeopleClientStd`] with the merged
//! [`Account`] the commands render through.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_people::v1::client::{PeopleClientStd, PeopleClientStdConnectOptions};
use secrecy::ExposeSecret;

use crate::{
    account::context::Account,
    config::{AccountConfig, Config},
};

/// The connected People client and the account it runs for.
pub struct PeopleClient {
    inner: PeopleClientStd,
    /// The merged account config the command runs against.
    pub account: Account,
}

impl Deref for PeopleClient {
    type Target = PeopleClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PeopleClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Opens the People client of the account, alongside its merged config.
///
/// Bails when the account has no `[people]` block.
pub fn build_people_client(
    config: Config,
    name: String,
    mut account_config: AccountConfig,
) -> Result<PeopleClient> {
    let people_config = account_config
        .people
        .take()
        .ok_or_else(|| anyhow!("Google People config is missing for account `{name}`"))?;

    let token = people_config.auth.token.get()?;
    let options = PeopleClientStdConnectOptions {
        tls: people_config.tls.into_tls(people_config.alpn),
    };
    let inner = PeopleClientStd::connect(token.expose_secret(), options)?;

    let account = Account::from(config).merge(Account::from(account_config));
    Ok(PeopleClient { inner, account })
}
