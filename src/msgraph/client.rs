//! # Microsoft Graph client
//!
//! Wraps [`io_msgraph::v1::client::MsgraphClientStd`] for the Graph
//! commands.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_msgraph::v1::client::{MsgraphClientStd, MsgraphClientStdConnectOptions};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::config::{AccountConfig, ProxyConfig};

/// The connected Graph client.
pub struct MsgraphClient {
    inner: MsgraphClientStd,
}

impl Deref for MsgraphClient {
    type Target = MsgraphClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MsgraphClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Opens the Graph client of the account.
///
/// Bails when the account declares no `[msgraph]` block.
pub fn build_msgraph_client(
    name: String,
    mut account_config: AccountConfig,
) -> Result<MsgraphClient> {
    let msgraph_config = account_config
        .msgraph
        .take()
        .ok_or_else(|| anyhow!("Microsoft Graph config is missing for account `{name}`"))?;

    let token = msgraph_config.auth.token.get()?;
    let options = MsgraphClientStdConnectOptions {
        tls: msgraph_config.tls.into_tls(msgraph_config.alpn),
        proxy: ProxyConfig::resolve(msgraph_config.proxy, &mut SecretResolver::new())?,
        user_id: msgraph_config.user_id,
    };
    let inner = MsgraphClientStd::connect(token.expose_secret(), options)?;

    Ok(MsgraphClient { inner })
}
