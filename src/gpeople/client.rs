//! # People client
//!
//! Wraps [`io_gpeople::v1::client::GpeopleClientStd`] with the merged
//! [`Account`] the commands render through.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_gpeople::v1::client::{GpeopleClientStd, GpeopleClientStdConnectOptions};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    account::context::Account,
    config::{AccountConfig, Config, ProxyConfig},
};

/// The connected People client and the account it runs for.
pub struct GpeopleClient {
    inner: GpeopleClientStd,
    /// The merged account config the command runs against.
    pub account: Account,
}

impl Deref for GpeopleClient {
    type Target = GpeopleClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for GpeopleClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Opens the People client of the account, alongside its merged config.
///
/// Bails when the account has no `[gpeople]` block.
pub fn build_gpeople_client(
    config: Config,
    name: String,
    mut account_config: AccountConfig,
) -> Result<GpeopleClient> {
    let gpeople_config = account_config
        .gpeople
        .take()
        .ok_or_else(|| anyhow!("Google People config is missing for account `{name}`"))?;

    let token = gpeople_config.auth.token.get()?;
    let options = GpeopleClientStdConnectOptions {
        tls: gpeople_config.tls.into_tls(gpeople_config.alpn),
        proxy: ProxyConfig::resolve(gpeople_config.proxy, &mut SecretResolver::new())?,
    };
    let inner = GpeopleClientStd::connect(token.expose_secret(), options)?;

    let account = Account::from(config).merge(Account::from(account_config));
    Ok(GpeopleClient { inner, account })
}
