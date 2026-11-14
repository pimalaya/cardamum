//! # Microsoft Graph client
//!
//! Wraps [`io_msgraph::v1::client::MsgraphClientStd`] for the Graph
//! commands.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_msgraph::v1::client::{
    MsgraphClientStd, MsgraphClientStdConnectOptions, MsgraphClientStdError,
};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    config::{AccountConfig, ProxyConfig},
    error::ErrorCode,
};

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

/// The code of a Graph refusal in an error chain, as neverest reads it:
/// Graph's `ErrorQuotaExceeded` or a 507 is `quota-exceeded`, a 429
/// `rate-limited`, a 403 `permission-denied`.
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    let MsgraphClientStdError::Send(send) = cause.downcast_ref::<MsgraphClientStdError>()? else {
        return None;
    };
    let err = send.api()?;

    match (err.status, err.code.as_str()) {
        (_, "ErrorQuotaExceeded") | (507, _) => Some(ErrorCode::QuotaExceeded),
        (429, _) => Some(ErrorCode::RateLimited),
        (403, _) => Some(ErrorCode::PermissionDenied),
        _ => None,
    }
}

/// Whether a contact request found no contact: a 404, or the 400 Graph
/// answers an id that cannot name one (`ErrorInvalidIdMalformed`).
pub(crate) fn is_missing(err: &MsgraphClientStdError) -> bool {
    let MsgraphClientStdError::Send(send) = err else {
        return false;
    };

    send.api().is_some_and(|err| {
        err.status == 404 || (err.status == 400 && err.code == "ErrorInvalidIdMalformed")
    })
}
