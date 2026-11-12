//! # People client
//!
//! Wraps [`io_gpeople::v1::client::GpeopleClientStd`] for the People
//! commands.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_gpeople::v1::client::{GpeopleClientStd, GpeopleClientStdConnectOptions};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::config::{AccountConfig, ProxyConfig};

/// The connected People client.
pub struct GpeopleClient {
    inner: GpeopleClientStd,
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

/// Opens the People client of the account.
///
/// Bails when the account has no `[gpeople]` block.
pub fn build_gpeople_client(
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

    Ok(GpeopleClient { inner })
}
