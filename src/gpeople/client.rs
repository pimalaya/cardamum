//! # People client
//!
//! Wraps [`io_gpeople::v1::client::GpeopleClientStd`] for the People
//! commands.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_gpeople::v1::{
    client::{GpeopleClientStd, GpeopleClientStdConnectOptions, GpeopleClientStdError},
    send::{GPEOPLE_API_BASE, GpeopleApiError, GpeopleSend, GpeopleSendError},
};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;
use serde_json::Value;
use url::Url;

use crate::{
    config::{AccountConfig, ProxyConfig},
    error::{ErrorCode, coded_cause},
    shared::refusal::Refusal,
};

/// The connected People client.
pub struct GpeopleClient {
    inner: GpeopleClientStd,
}

#[cfg(test)]
impl GpeopleClient {
    /// Wraps an io-gpeople client, as a test drives one over a script.
    pub fn over(inner: GpeopleClientStd) -> Self {
        Self { inner }
    }
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

/// The code of a People refusal in an error chain ([`google_code`]).
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    match cause.downcast_ref::<GpeopleClientStdError>()? {
        GpeopleClientStdError::Send(GpeopleSendError::Api(err)) => google_code(err),
        _ => None,
    }
}

/// The refusal of a People error in a chain, for the `gpeople` commands:
/// the status, the first reason (else the canonical status) and the
/// message of what People answered.
pub(crate) fn refusal(cause: &(dyn std::error::Error + 'static)) -> Option<Refusal> {
    let err = match cause.downcast_ref::<GpeopleClientStdError>() {
        Some(GpeopleClientStdError::Send(err)) => err,
        Some(_) => return None,
        None => cause.downcast_ref::<GpeopleSendError>()?,
    };
    let err = err.api()?;
    let reason = err.reasons.first().or(err.google_status.as_ref());

    Some(Refusal {
        status: Some(err.status),
        reason: reason.cloned(),
        message: err.message.clone(),
        code: None,
        retry_after: None,
    })
}

impl GpeopleClient {
    /// The resource `<collection>/<id>`, queried by `pairs`, as People
    /// sent it, the id one percent-encoded segment.
    pub fn raw_get(
        &mut self,
        collection: &str,
        id: &str,
        pairs: &[(&str, String)],
    ) -> Result<Value, GpeopleClientStdError> {
        if matches!(id, "" | "." | "..") {
            let err = format!("Resource name {collection}/{id:?} names no resource");
            return Err(GpeopleSendError::InvalidRequest(err).into());
        }

        let mut url = Url::parse(GPEOPLE_API_BASE).map_err(GpeopleSendError::from)?;
        url.path_segments_mut()
            .map_err(|()| GpeopleSendError::InvalidRequest(GPEOPLE_API_BASE.into()))?
            .pop_if_empty()
            .push(collection)
            .push(id);
        url.query_pairs_mut()
            .extend_pairs(pairs.iter().map(|(key, value)| (*key, value.as_str())));

        let auth = self.auth.clone();
        Ok(self.run(GpeopleSend::<Value>::get(&auth, url))?.response)
    }
}

/// The failure of a request naming one person, coded `card-not-found`
/// when People holds no such person, its message People's: the specific
/// commands code it as the unified ones do.
pub(crate) fn person_refusal(err: GpeopleClientStdError) -> anyhow::Error {
    match is_missing(&err) {
        true => coded_cause(err, ErrorCode::CardNotFound),
        false => err.into(),
    }
}

/// Whether People refused to create a contact group because the user
/// holds one of that name already: the 409 contactGroups.create answers a
/// duplicate name.
pub(crate) fn is_group_taken(err: &GpeopleClientStdError) -> bool {
    match err {
        GpeopleClientStdError::Send(send) => send.status() == Some(409),
        _ => false,
    }
}

/// Whether a person request found no person: 404, or 410 for one gone.
pub(crate) fn is_missing(err: &GpeopleClientStdError) -> bool {
    match err {
        GpeopleClientStdError::Send(send) => matches!(send.status(), Some(404 | 410)),
        _ => false,
    }
}

/// The code of a Google refusal, neverest's (its src/throttle.rs): a 429,
/// or a 403 naming a rate limit or a quota, is `rate-limited`; any other
/// 403 `permission-denied`. A 412, or a `FAILED_PRECONDITION` naming no
/// expired sync token, is `precondition-failed`: People answers the
/// stale `etag` of an update so (400 `FAILED_PRECONDITION`).
fn google_code(err: &GpeopleApiError) -> Option<ErrorCode> {
    match err.status {
        429 => Some(ErrorCode::RateLimited),
        403 if names_rate_limit(err) => Some(ErrorCode::RateLimited),
        403 => Some(ErrorCode::PermissionDenied),
        412 => Some(ErrorCode::PreconditionFailed),
        _ if names_stale_version(err) => Some(ErrorCode::PreconditionFailed),
        _ => None,
    }
}

/// Whether a Google refusal says the version a write carries is stale:
/// `FAILED_PRECONDITION` or the `failedPrecondition` reason, unless it
/// names an expired sync token, refused the same way.
fn names_stale_version(err: &GpeopleApiError) -> bool {
    let failed = err.google_status.as_deref() == Some("FAILED_PRECONDITION")
        || err
            .reasons
            .iter()
            .any(|reason| reason == "failedPrecondition");
    let expired = err
        .detail_reasons
        .iter()
        .any(|reason| reason == "EXPIRED_SYNC_TOKEN")
        || squeeze(&err.message).contains("synctoken");

    failed && !expired
}

/// Whether a Google refusal names a rate limit or a quota, by reason or
/// by message: the 403 neverest's `google_refusal` reads as a daily
/// quota, a per-minute one or a burst.
fn names_rate_limit(err: &GpeopleApiError) -> bool {
    let has_reason = |reason: &str| {
        err.reasons
            .iter()
            .chain(&err.detail_reasons)
            .any(|r| r == reason)
    };

    let mut text = err.message.clone();
    for reason in err.reasons.iter().chain(&err.detail_reasons) {
        text.push(' ');
        text.push_str(reason);
    }
    let squeezed = squeeze(&text);

    let daily = has_reason("dailyLimitExceeded")
        || ((squeezed.contains("dailylimit") || squeezed.contains("perday"))
            && !squeezed.contains("unauthenticated"));
    let minute = squeezed.contains("perminute") || squeezed.contains("per100seconds");
    let rate = err.google_status.as_deref() == Some("RESOURCE_EXHAUSTED")
        || [
            "rateLimitExceeded",
            "userRateLimitExceeded",
            "quotaExceeded",
            "RATE_LIMIT_EXCEEDED",
        ]
        .into_iter()
        .any(has_reason)
        || squeezed.contains("ratelimitexceeded")
        || squeezed.contains("quotaexceeded");

    daily || minute || rate
}

/// The ASCII letters and digits of a text, lowercased.
fn squeeze(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use io_gpeople::v1::client::GpeopleClientStd;
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::json;

    use crate::{
        error::{self, ErrorCode},
        gpeople::{
            client::GpeopleClient,
            connection::{
                delete::GpeopleConnectionDeleteCommand, get::GpeopleConnectionGetCommand,
            },
        },
        script::{self, Script},
    };

    /// The client over a People server answering `status` with `message`.
    fn refusing(status: u16, message: &str) -> GpeopleClient {
        let body = json!({"error": {"code": status, "message": message}}).to_string();
        let (script, _) = Script::new([script::response(status, "application/json", &body)]);
        let inner = GpeopleClientStd::new(script, "token");
        GpeopleClient { inner }
    }

    /// A specific command naming a person People does not hold carries
    /// `card-not-found`, as `card read` does, People's message kept.
    #[test]
    fn a_specific_command_on_a_missing_person_is_coded() {
        let printer = &mut CapturePrinter::default();
        let missing = Some(ErrorCode::CardNotFound);
        let found = "Requested entity was not found.";

        let get = GpeopleConnectionGetCommand {
            person_id: String::from("c9"),
        };
        let err = get.execute(printer, refusing(404, found)).unwrap_err();
        assert_eq!(error::code(&err), missing, "{err:#}");
        assert!(format!("{err:#}").contains(found), "{err:#}");

        let delete = GpeopleConnectionDeleteCommand {
            person_id: String::from("c9"),
        };
        let err = delete.execute(printer, refusing(404, found)).unwrap_err();
        assert_eq!(error::code(&err), missing, "{err:#}");
    }

    /// A People listing lifts `nextPageToken` into `nextPage`, `null` on
    /// the last page, the sync token staying as People sent it, and
    /// `--page-token` sends `nextPage` back as the `pageToken`.
    #[test]
    fn a_people_listing_pages_by_next_page() {
        use clap::Parser;
        use serde_json::Value;

        use crate::gpeople::{
            connection::list::GpeopleConnectionListCommand,
            contact_group::list::GpeopleContactGroupListCommand,
            other_contact::list::GpeopleOtherContactListCommand,
        };

        let run = |args: &[&str], body: Value| {
            let response = script::response(200, "application/json", &body.to_string());
            let (script, requests) = Script::new([response]);
            let client = GpeopleClient {
                inner: GpeopleClientStd::new(script, "token"),
            };
            let mut printer = CapturePrinter::default();

            match args[0] {
                "connections" => GpeopleConnectionListCommand::parse_from(args)
                    .execute(&mut printer, client)
                    .unwrap(),
                "other-contacts" => GpeopleOtherContactListCommand::parse_from(args)
                    .execute(&mut printer, client)
                    .unwrap(),
                "contact-groups" => GpeopleContactGroupListCommand::parse_from(args)
                    .execute(&mut printer, client)
                    .unwrap(),
                other => panic!("{other}"),
            }

            let request = String::from_utf8_lossy(&requests.lock().unwrap()[0]).into_owned();
            (printer.into_single().unwrap(), request)
        };

        for (command, key) in [
            ("connections", "connections"),
            ("other-contacts", "otherContacts"),
            ("contact-groups", "contactGroups"),
        ] {
            let first = json!({key: [], "nextPageToken": "t2"});
            let (json, _) = run(&[command], first);
            assert_eq!(json["nextPage"], "t2", "{json}");
            assert!(json.get("nextPageToken").is_none(), "{json}");

            let last = json!({key: [], "nextSyncToken": "s1"});
            let (json, request) = run(&[command, "--page-token", "t2"], last);
            assert!(request.contains("pageToken=t2"), "{request}");
            assert_eq!(json["nextPage"], Value::Null, "{json}");
            assert_eq!(json["nextSyncToken"], "s1", "{json}");
        }
    }
}
