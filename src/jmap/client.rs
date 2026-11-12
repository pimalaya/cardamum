//! # JMAP client
//!
//! Wraps [`JmapClientStd`], its JMAP session already discovered.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_jmap::client::{JmapClientStd, JmapClientStdConnectOptions};
use pimalaya_config::secret::SecretResolver;

use crate::{
    config::{AccountConfig, ProxyConfig, parse_server},
    jmap::backend::jmap_http_auth,
};

/// A connected io-jmap client.
pub struct JmapClient {
    inner: JmapClientStd,
}

impl Deref for JmapClient {
    type Target = JmapClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for JmapClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Opens the JMAP client and fetches its session.
///
/// Bails when the account declares no `[jmap]` block.
pub fn build_jmap_client(name: String, mut account_config: AccountConfig) -> Result<JmapClient> {
    let jmap_config = account_config
        .jmap
        .take()
        .ok_or_else(|| anyhow!("JMAP config is missing for account `{name}`"))?;

    let mut resolver = SecretResolver::new();
    let http_auth = jmap_http_auth(jmap_config.auth, &mut resolver)?;
    let url = parse_server(
        &jmap_config.server,
        "https",
        &["http", "https", "jmap", "jmaps"],
    )?;
    let options = JmapClientStdConnectOptions {
        tls: jmap_config.tls.into_tls(jmap_config.alpn),
        proxy: ProxyConfig::resolve(jmap_config.proxy, &mut resolver)?,
    };

    let mut inner = JmapClientStd::connect(&url, http_auth, options)?;
    inner.session_get(&url)?;

    Ok(JmapClient { inner })
}
