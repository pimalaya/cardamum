//! Cardamum wrapper around [`io_msgraph::v1::client::MsgraphClientStd`]
//! that bundles the merged [`Account`] alongside the connected client.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_msgraph::v1::client::{MsgraphClientStd, MsgraphClientStdConnectOptions};
use secrecy::ExposeSecret;

use crate::{
    account::context::Account,
    config::{AccountConfig, Config},
};

pub struct MsgraphClient {
    inner: MsgraphClientStd,
    pub account: Account,
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

/// Builds the merged [`Account`] from the already-resolved config and
/// account, then opens the Graph client. Bails when the account has no
/// `[msgraph]` block.
pub fn build_msgraph_client(
    config: Config,
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
        user_id: msgraph_config.user_id,
    };
    let inner = MsgraphClientStd::connect(token.expose_secret(), options)?;

    let account = Account::from(config).merge(Account::from(account_config));
    Ok(MsgraphClient { inner, account })
}
