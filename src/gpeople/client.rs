//! # People client
//!
//! Wraps [`io_gpeople::v1::client::GpeopleClientStd`] for the People
//! commands.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, anyhow};
use io_gpeople::v1::{
    client::{GpeopleClientStd, GpeopleClientStdConnectOptions, GpeopleClientStdError},
    send::{GpeopleApiError, GpeopleSendError},
};
use pimalaya_cli::error::with_code;
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    config::{AccountConfig, ProxyConfig},
    error::ErrorCode,
};

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

/// The code of a People refusal in an error chain ([`google_code`]).
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    match cause.downcast_ref::<GpeopleClientStdError>()? {
        GpeopleClientStdError::Send(GpeopleSendError::Api(err)) => google_code(err),
        _ => None,
    }
}

/// The failure of a request naming one person, coded `card-not-found`
/// when People holds no such person, its message People's: the specific
/// commands code it as the unified ones do.
pub(crate) fn person_refusal(err: GpeopleClientStdError) -> anyhow::Error {
    match is_missing(&err) {
        true => with_code(err.into(), |_| Some(ErrorCode::CardNotFound)),
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
/// 403 `permission-denied`.
fn google_code(err: &GpeopleApiError) -> Option<ErrorCode> {
    match err.status {
        429 => Some(ErrorCode::RateLimited),
        403 if names_rate_limit(err) => Some(ErrorCode::RateLimited),
        403 => Some(ErrorCode::PermissionDenied),
        _ => None,
    }
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
