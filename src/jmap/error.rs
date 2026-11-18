//! # Set error rendering
//!
//! Renders the per-item errors of a JMAP write (RFC 8620 §5.3) as prose
//! a user can read, rather than as the `Debug` shape of the error value.
//!
//! Each rejected item carries its own typed error, with an optional
//! server description and, for `invalidProperties`, the property names
//! at fault. Mirrors himalaya's jmap/error.rs.

use std::fmt;

use anyhow::Error;
use io_jmap::{
    client::JmapClientStdError,
    rfc8620::{
        error::{JmapMethodError, JmapRequestError},
        send::JmapSendError,
    },
    rfc9610::{
        address_book::set::JmapAddressBookSetItemError,
        contact_card::{copy::JmapContactCardCopyItemError, set::JmapContactCardSetItemError},
    },
};
use pimalaya_cli::sanitize;
use serde::Serialize;

use crate::{
    error::{ErrorCode, ItemError},
    shared::refusal::Refusal,
};

/// The parts of a JMAP set error worth showing.
pub trait JmapSetError {
    /// The error type as JMAP spells it, e.g. `invalidProperties`.
    fn type_name(&self) -> &str;
    /// The optional human-readable detail the server sent along.
    fn description(&self) -> Option<&str>;
    /// The property names at fault, empty outside `invalidProperties`.
    fn properties(&self) -> &[String];
    /// The code of a `notFound`, naming what the set found missing.
    fn not_found(&self) -> ErrorCode;

    /// The code of the error, read from its type (RFC 8620 §5.3): none
    /// for a type no caller acts on, its type kept in the message.
    fn code(&self) -> Option<ErrorCode> {
        match self.type_name() {
            "forbidden" => Some(ErrorCode::PermissionDenied),
            "notFound" => Some(self.not_found()),
            "overQuota" => Some(ErrorCode::QuotaExceeded),
            "rateLimit" => Some(ErrorCode::RateLimited),
            _ => None,
        }
    }
}

/// Renders a set error as the suffix of a rejection message.
pub fn format_set_error<E: JmapSetError>(err: &E) -> String {
    let mut msg = format!(": {}", err.type_name());

    if !err.properties().is_empty() {
        msg.push_str(" (`");
        msg.push_str(&err.properties().join("`, `"));
        msg.push_str("`)");
    }

    if let Some(description) = err.description() {
        msg.push_str(": ");
        msg.push_str(description.trim_end_matches(['.', '\n']));
    }

    msg
}

/// The failure of a write the server refused, `prefix` naming the write,
/// followed by the rendered error ([`format_set_error`]) and coded by its
/// type.
pub fn set_error<E: JmapSetError>(prefix: &str, err: &E) -> Error {
    set_error_message(format!("{prefix}{}", format_set_error(err)), err)
}

/// The failure of a write the server refused, worded `message`, coded by
/// the type of `err`, its SetError kept for the refusal a `jmap` command
/// prints ([`refusal`]).
pub fn set_error_message<E: JmapSetError>(message: String, err: &E) -> Error {
    JmapSetRefusal {
        message,
        type_name: err.type_name().to_owned(),
        description: err.description().map(ToOwned::to_owned),
        code: err.code(),
    }
    .into()
}

/// The one write of a whole command the server refused, worded by the
/// command, its SetError kept for the code and the refusal it reads as.
#[derive(Debug)]
pub struct JmapSetRefusal {
    message: String,
    type_name: String,
    description: Option<String>,
    code: Option<ErrorCode>,
}

impl fmt::Display for JmapSetRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for JmapSetRefusal {}

/// The code of a JMAP refusal in an error chain: a refused write by its
/// SetError type, an HTTP 429 `rate-limited` and a 403
/// `permission-denied`, as is a method error `forbidden` (RFC 8620
/// §3.6.2).
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    if let Some(err) = cause.downcast_ref::<JmapSetRefusal>() {
        return err.code;
    }

    if let Some(JmapMethodError::Forbidden { .. }) = cause.downcast_ref::<JmapMethodError>() {
        return Some(ErrorCode::PermissionDenied);
    }

    match send_error(cause)? {
        JmapSendError::HttpStatus(status) | JmapSendError::Request { status, .. } => match status {
            429 => Some(ErrorCode::RateLimited),
            403 => Some(ErrorCode::PermissionDenied),
            _ => None,
        },
        _ => None,
    }
}

/// The refusal of a JMAP error in a chain, for the `jmap` commands: a
/// non-2xx answer by its status (and its problem type, for a request the
/// server refused whole), a method error or a refused write by its type
/// and description.
pub(crate) fn refusal(cause: &(dyn std::error::Error + 'static)) -> Option<Refusal> {
    if let Some(err) = cause.downcast_ref::<JmapSetRefusal>() {
        let message = err
            .description
            .clone()
            .unwrap_or_else(|| err.message.clone());
        return Some(described(None, Some(err.type_name.clone()), message));
    }

    if let Some(err) = cause.downcast_ref::<JmapMethodError>() {
        return Some(typed(err));
    }

    match send_error(cause)? {
        JmapSendError::Request { status, error } => Some(request_refusal(*status, error)),
        JmapSendError::HttpStatus(status) => {
            Some(described(Some(*status), None, cause.to_string()))
        }
        _ => None,
    }
}

/// The send error of a chain cause: one met by a typed call, or the one a
/// raw send ends with, which io-jmap's client wraps transparently.
fn send_error<'a>(cause: &'a (dyn std::error::Error + 'static)) -> Option<&'a JmapSendError> {
    match cause.downcast_ref::<JmapClientStdError>() {
        Some(JmapClientStdError::Send(err)) => Some(err),
        Some(_) => None,
        None => cause.downcast_ref::<JmapSendError>(),
    }
}

/// The refusal of a request the server refused whole: its problem type,
/// the URN namespace stripped, and its detail.
fn request_refusal(status: u16, err: &JmapRequestError) -> Refusal {
    let (reason, detail) = match err {
        JmapRequestError::UnknownCapability { detail } => ("unknownCapability", detail),
        JmapRequestError::NotJson { detail } => ("notJSON", detail),
        JmapRequestError::NotRequest { detail } => ("notRequest", detail),
        JmapRequestError::Limit { detail, .. } => ("limit", detail),
        JmapRequestError::Other { type_name, detail } => (type_name.as_str(), detail),
    };
    let reason = reason.trim_start_matches("urn:ietf:params:jmap:error:");
    let message = detail.clone().unwrap_or_else(|| err.to_string());

    described(Some(status), Some(reason.to_owned()), message)
}

/// The refusal a method error states: its `type`, then its
/// `description`, the type standing in for a missing one.
fn typed(err: &impl Serialize) -> Refusal {
    let value = serde_json::to_value(err).unwrap_or_default();
    let field = |key: &str| value.get(key).and_then(|value| value.as_str());
    let reason = field("type");
    let message = field("description").or(reason).unwrap_or_default();

    described(None, reason.map(ToOwned::to_owned), message.to_owned())
}

fn described(status: Option<u16>, reason: Option<String>, message: String) -> Refusal {
    Refusal {
        status,
        reason,
        message,
        ..Default::default()
    }
}

/// The item error of the object `id` a set refused, coded by its type,
/// the type kept in `sources`.
pub fn item_error<E: JmapSetError>(id: &str, err: &E) -> ItemError {
    ItemError {
        id: Some(id.to_owned()),
        code: err.code(),
        error: format!("JMAP server refused the write{}", format_set_error(err)),
        sources: vec![sanitize::text(err.type_name()).into_owned()],
    }
}

impl JmapSetError for JmapAddressBookSetItemError {
    fn not_found(&self) -> ErrorCode {
        ErrorCode::AddressbookNotFound
    }

    fn type_name(&self) -> &str {
        match self {
            Self::AddressBookHasContents { .. } => "addressBookHasContents",
            Self::Forbidden { .. } => "forbidden",
            Self::NotFound { .. } => "notFound",
            Self::InvalidPatch { .. } => "invalidPatch",
            Self::WillDestroy { .. } => "willDestroy",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Other { type_name, .. } if type_name.is_empty() => "unknown",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::AddressBookHasContents { description }
            | Self::Forbidden { description }
            | Self::NotFound { description }
            | Self::InvalidPatch { description }
            | Self::WillDestroy { description }
            | Self::InvalidProperties { description, .. }
            | Self::Other { description, .. } => description.as_deref(),
        }
    }

    fn properties(&self) -> &[String] {
        match self {
            Self::InvalidProperties { properties, .. } => properties,
            _ => &[],
        }
    }
}

impl JmapSetError for JmapContactCardSetItemError {
    fn not_found(&self) -> ErrorCode {
        ErrorCode::CardNotFound
    }

    fn type_name(&self) -> &str {
        match self {
            Self::BlobNotFound { .. } => "blobNotFound",
            Self::Forbidden { .. } => "forbidden",
            Self::NotFound { .. } => "notFound",
            Self::InvalidPatch { .. } => "invalidPatch",
            Self::WillDestroy { .. } => "willDestroy",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Other { type_name, .. } if type_name.is_empty() => "unknown",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::BlobNotFound { description }
            | Self::Forbidden { description }
            | Self::NotFound { description }
            | Self::InvalidPatch { description }
            | Self::WillDestroy { description }
            | Self::InvalidProperties { description, .. }
            | Self::Other { description, .. } => description.as_deref(),
        }
    }

    fn properties(&self) -> &[String] {
        match self {
            Self::InvalidProperties { properties, .. } => properties,
            _ => &[],
        }
    }
}

impl JmapSetError for JmapContactCardCopyItemError {
    fn not_found(&self) -> ErrorCode {
        ErrorCode::CardNotFound
    }

    fn type_name(&self) -> &str {
        match self {
            Self::AlreadyExists { .. } => "alreadyExists",
            Self::NotFound { .. } => "notFound",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Other { type_name, .. } if type_name.is_empty() => "unknown",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::AlreadyExists { description, .. }
            | Self::NotFound { description }
            | Self::InvalidProperties { description, .. }
            | Self::Other { description, .. } => description.as_deref(),
        }
    }

    fn properties(&self) -> &[String] {
        match self {
            Self::InvalidProperties { properties, .. } => properties,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_type_the_properties_and_the_description() {
        let err = JmapContactCardSetItemError::InvalidProperties {
            description: Some("unknown property.".into()),
            properties: vec!["vCardProps".into(), "links/1/@type".into()],
        };

        assert_eq!(
            format_set_error(&err),
            ": invalidProperties (`vCardProps`, `links/1/@type`): unknown property"
        );
    }

    #[test]
    fn renders_an_error_carrying_neither_properties_nor_description() {
        let err = JmapContactCardSetItemError::NotFound { description: None };

        assert_eq!(format_set_error(&err), ": notFound");
    }

    /// The code a refused write of a card answered `set_error` fails with,
    /// the error read as io-jmap reads it off the wire.
    fn card_code(set_error: serde_json::Value) -> Option<ErrorCode> {
        let err: JmapContactCardSetItemError = serde_json::from_value(set_error).unwrap();
        let err = super::set_error("ContactCard destroy rejected", &err);
        crate::error::code(&err)
    }

    #[test]
    fn a_set_error_is_coded_by_its_type() {
        use serde_json::json;

        let code = |kind: &str| card_code(json!({"type": kind}));

        assert_eq!(code("forbidden"), Some(ErrorCode::PermissionDenied));
        assert_eq!(code("notFound"), Some(ErrorCode::CardNotFound));
        assert_eq!(code("overQuota"), Some(ErrorCode::QuotaExceeded));
        assert_eq!(code("rateLimit"), Some(ErrorCode::RateLimited));
        assert_eq!(code("invalidPatch"), None);
        assert_eq!(code("tooLarge"), None);
    }

    /// An address book a set does not find is no card.
    #[test]
    fn an_address_book_not_found_is_coded_so() {
        let err: JmapAddressBookSetItemError =
            serde_json::from_value(serde_json::json!({"type": "notFound"})).unwrap();
        let err = set_error("AddressBook destroy rejected", &err);

        assert_eq!(
            crate::error::code(&err),
            Some(ErrorCode::AddressbookNotFound)
        );
        assert_eq!(err.to_string(), "AddressBook destroy rejected: notFound");
    }
}
