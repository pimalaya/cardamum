//! Cardamum wrapper around [`io_jmap::client::JmapClientStd`] that
//! bundles the merged [`Account`] alongside the connected client (its
//! JMAP session already discovered).

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_jmap::client::JmapClientStd;

use crate::{
    account::context::Account,
    config::{AccountConfig, Config, parse_server},
    jmap::backend::jmap_http_auth,
};

pub struct JmapClient {
    inner: JmapClientStd,
    pub account: Account,
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

/// Builds the merged [`Account`] from the already-resolved config and
/// account, then opens the JMAP client and fetches its session. Bails
/// when the account has no `[jmap]` block.
pub fn build_jmap_client(
    config: Config,
    name: String,
    mut account_config: AccountConfig,
) -> Result<JmapClient> {
    let jmap_config = account_config
        .jmap
        .take()
        .ok_or_else(|| anyhow!("JMAP config is missing for account `{name}`"))?;

    let tls = jmap_config.tls.into_tls(jmap_config.alpn);
    let http_auth = jmap_http_auth(jmap_config.auth)?;
    let url = parse_server(
        &jmap_config.server,
        "https",
        &["http", "https", "jmap", "jmaps"],
    )?;

    let mut inner = JmapClientStd::connect(&url, &tls, http_auth)?;
    inner.session_get(&url)?;

    let account = Account::from(config).merge(Account::from(account_config));
    Ok(JmapClient { inner, account })
}
