//! # Error codes
//!
//! A stable, machine-readable name for the failures a caller is expected
//! to act on, printed as `code` beside the message under `--json`.
//!
//! The message stays for humans and may be reworded at any time; the
//! code is the contract. A failure with no code prints `"code": null`.
//! The mechanism (`CodedError`, `code_of`, the report) lives in
//! `pimalaya_cli::error`; only the vocabulary is cardamum's.
//!
//! A server's refusal takes a code of the vocabulary Himalaya shares,
//! `permission-denied`, `quota-exceeded` and `rate-limited`, read from
//! the typed error of the protocol that answered it ([`code`]): one
//! reader per backend walks the error chain, which keeps the typed
//! errors the backends return. cardamum retries nothing, so
//! `rate-limited` codes the first throttled answer met. The account a
//! command names, or the default one it takes, is coded the same way,
//! read from the `AccountError` pimalaya-config returns.

use anyhow::Error;
#[cfg(feature = "pimdir")]
use io_pimdir::client::PimdirError;
use pimalaya_cli::error::{PartialOutcome, code_of, with_code};
use pimalaya_config::toml::AccountError;
use schemars::JsonSchema;
use serde::Serialize;

/// The stable codes, each serialised in kebab-case.
///
/// Declared in every build, the pimdir ones included, so the
/// `cardamum-error` schema a caller reads does not change with the
/// backends compiled in.
// NOTE: a build leaves out the backends raising some codes.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// No account of the configuration has the name `-a` gives.
    AccountMissing,
    /// No `-a` names an account and none is marked `default = true`.
    AccountDefaultMissing,
    /// No `-a` names an account and several are marked `default = true`.
    AccountDefaultSeveral,
    /// The backend the account resolves to is configured but left out of
    /// this build: the message names the cargo feature to rebuild with.
    BackendNotBuilt,
    /// A command would read its input off stdin inside a session, where
    /// stdin carries the commands: the caller passes it as an argument or
    /// a file path.
    StdinTaken,
    /// The backend the account resolves to does not serve a capability
    /// the command reaches, refused before connecting: the caller picks
    /// another backend or leaves the option out.
    CapabilityMissing,
    /// The server refused the request to the account: an HTTP 403 that is
    /// no rate limit and, on CardDAV, names no precondition but
    /// `need-privileges` (RFC 3744 §7.1.1), or a JMAP `forbidden`.
    PermissionDenied,
    /// The write would exceed the account's quota: an HTTP 507, a CardDAV
    /// `quota-not-exceeded` (RFC 4331 §6), Graph's `ErrorQuotaExceeded`
    /// or a JMAP `overQuota`.
    QuotaExceeded,
    /// The server throttled the request: an HTTP 429, a People 403
    /// naming a rate limit or a quota, or a JMAP `rateLimit`. cardamum
    /// retries nothing, so the caller waits before trying again.
    RateLimited,
    /// The card a command names is not in the address book.
    CardNotFound,
    /// The address book a command names does not exist: nothing was read
    /// or written.
    AddressbookNotFound,
    /// The backend answered for a card cardamum could not read.
    CardUnreadable,
    /// The card is listed but its body is not local yet: a sync will
    /// bring it, so the caller waits rather than reports a loss.
    BodyPending,
    /// A write named, by `--if-match`, a version the card no longer has:
    /// the caller reads it again rather than overwrites a change.
    PreconditionFailed,
    /// The pimdir root holds no store: the sync engine owning it creates
    /// it on its first run.
    StoreUncreated,
    /// The pimdir store is of another format version than the one this
    /// cardamum reads, which a store opens under alone: one an earlier
    /// neverest wrote is reinstalled, a later one asks for a cardamum
    /// built on it.
    StoreVersion,
    /// The pimdir root holds a database another program wrote, which no
    /// pimdir tool reads or writes.
    StoreForeign,
    /// The pimdir store's recorded version disagrees with itself, or its
    /// schema differs from the one its version declares.
    StoreCorrupt,
    /// The pimdir store names its bodies by a hash this cardamum does not
    /// know.
    StoreHashAlgo,
    /// Another user can read or write the pimdir store's directory or
    /// database, or a link stands where a lock file belongs (pimdir
    /// STORAGE §18): nothing was opened, and the user restricts it.
    StoreExposed,
    /// Another writer held the pimdir store: past the wait on a write, at
    /// once on a check, which does not wait. The caller tries again.
    StoreBusy,
    /// The store's staging lock file was replaced while a write held it
    /// (pimdir STORAGE §8): nothing was queued, and the caller tries
    /// again once no other process is setting the store up.
    LockReplaced,
    /// A source syncing the address book does not support the write
    /// (pimdir STORAGE §15.6): nothing was queued.
    Unsupported,
    /// A create names a card the address book already holds, live or
    /// queued: the caller updates that card instead.
    ItemDuplicate,
    /// A queued action's payload is malformed (pimdir STORAGE §15.3): the
    /// store's owner parks it on its next drain.
    ActionMalformed,
    /// The pimdir store has no search index yet: its sync engine builds
    /// it.
    IndexMissing,
    /// The pimdir search index was built otherwise or from another store:
    /// its sync engine rebuilds it.
    IndexOutdated,
    /// The pimdir search index is of another draft or shape: its sync
    /// engine rebuilds one of an earlier draft, and leaves any other.
    IndexVersion,
    /// The SQLite cardamum links lacks what the pimdir search index needs
    /// (FTS5, 3.43 or later).
    IndexFts5Missing,
    /// The pimdir search query is malformed or names an unknown field or
    /// value (pimdir SEARCH §8.4): nothing was searched.
    QueryRefused,
    /// The pimdir search index could not answer an accepted query, an
    /// implementation fault.
    QueryUnanswerable,
}

#[cfg(feature = "pimdir")]
impl ErrorCode {
    /// The code of a pimdir failure, read from the one io-pimdir names
    /// for it ([`PimdirError::code`]), or none for a failure no caller
    /// acts on (SQL, I/O, JSON) or no cardamum path raises.
    pub fn of_pimdir(err: &PimdirError) -> Option<Self> {
        match err.code() {
            "store-uncreated" => Some(Self::StoreUncreated),
            "store-version" => Some(Self::StoreVersion),
            "store-foreign" => Some(Self::StoreForeign),
            "store-corrupt" => Some(Self::StoreCorrupt),
            "store-hash-algo" => Some(Self::StoreHashAlgo),
            "store-exposed" => Some(Self::StoreExposed),
            "store-busy" => Some(Self::StoreBusy),
            "lock-replaced" => Some(Self::LockReplaced),
            "unsupported" => Some(Self::Unsupported),
            "item-duplicate" => Some(Self::ItemDuplicate),
            "action-malformed" => Some(Self::ActionMalformed),
            "index-missing" => Some(Self::IndexMissing),
            "index-outdated" => Some(Self::IndexOutdated),
            "index-version" => Some(Self::IndexVersion),
            "index-fts5-missing" => Some(Self::IndexFts5Missing),
            "query-refused" => Some(Self::QueryRefused),
            "query-unanswerable" => Some(Self::QueryUnanswerable),
            _ => None,
        }
    }
}
/// The code of `err`: the one a coded error in its chain carries, else
/// the one a typed error under it names.
pub fn code(err: &Error) -> Option<ErrorCode> {
    code_of(err).or_else(|| typed_code(err))
}

/// A reader of one kind of typed errors, coding those it knows.
type Reader = fn(&(dyn std::error::Error + 'static)) -> Option<ErrorCode>;

/// The code a typed error of `err`'s chain names, the account taken or
/// the refusal of the backend that answered: the first cause a reader
/// codes.
fn typed_code(err: &Error) -> Option<ErrorCode> {
    let readers: &[Reader] = &[
        account_code,
        #[cfg(feature = "carddav")]
        crate::carddav::client::refusal_code,
        #[cfg(feature = "gpeople")]
        crate::gpeople::client::refusal_code,
        #[cfg(feature = "msgraph")]
        crate::msgraph::client::refusal_code,
        #[cfg(feature = "vdir")]
        crate::vdir::client::missing_code,
    ];

    err.chain()
        .find_map(|cause| readers.iter().find_map(|reader| reader(cause)))
}

/// The code of the account `-a` names, or the default one, not taken.
fn account_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    match cause.downcast_ref::<AccountError>()? {
        AccountError::Missing { .. } => Some(ErrorCode::AccountMissing),
        AccountError::NoDefault => Some(ErrorCode::AccountDefaultMissing),
        AccountError::SeveralDefaults(_) => Some(ErrorCode::AccountDefaultSeveral),
    }
}

/// The error of one item a listing could not read.
pub type ItemError = pimalaya_cli::error::ItemError<ErrorCode>;

/// The entry of the card `id` a listing could not read, `err` saying why:
/// coded as `err` is, else `card-unreadable`.
pub fn unreadable_card(id: Option<&str>, err: Error) -> ItemError {
    let mut entry = ItemError::new(id, &with_code(err, code));
    entry.code = entry.code.or(Some(ErrorCode::CardUnreadable));
    entry
}

/// Ends a command that printed its result: exit 3 when an item failed.
pub fn partial_when(errors: &[ItemError]) -> anyhow::Result<()> {
    match errors.is_empty() {
        true => Ok(()),
        false => Err(PartialOutcome::default().into()),
    }
}

/// The failure of the card `card_id` the address book does not hold,
/// `cause` what the server answered.
#[cfg(any(feature = "carddav", feature = "gpeople", feature = "msgraph"))]
pub fn card_not_found(card_id: &str, cause: Error) -> Error {
    with_code(cause, |_| Some(ErrorCode::CardNotFound))
        .context(format!("Card `{card_id}` not found"))
}

#[cfg(test)]
mod account_tests {
    use super::*;

    /// Every way of not taking an account carries its own code, read off
    /// the error pimalaya-config returns as `resolve_account` passes it.
    #[test]
    fn an_account_not_taken_is_coded() {
        let failures = [
            (
                AccountError::Missing {
                    name: "default".into(),
                    names: vec!["work".into()],
                },
                ErrorCode::AccountMissing,
            ),
            (AccountError::NoDefault, ErrorCode::AccountDefaultMissing),
            (
                AccountError::SeveralDefaults(vec!["a".into(), "b".into()]),
                ErrorCode::AccountDefaultSeveral,
            ),
        ];

        for (err, expected) in failures {
            let err = Error::from(err);
            assert_eq!(code(&err), Some(expected));
            assert_eq!(code_of(&with_code(err, code)), Some(expected));
        }
    }
}

#[cfg(all(test, feature = "pimdir"))]
mod tests {
    use io_pimdir::{
        capability::PimdirRefusal,
        client::{PimdirDuplicate, PimdirError},
        codec::PimdirActionError,
        search::query::PimdirQuery,
    };

    use super::*;

    /// Every pimdir failure a cardamum path raises and a caller acts on
    /// prints the very code io-pimdir names for it, so a script reads one
    /// vocabulary whichever program met the store.
    #[test]
    fn a_pimdir_failure_prints_the_code_io_pimdir_names() {
        let failures = [
            PimdirError::Uncreated,
            PimdirError::Version {
                found: Some("storage-draft-10".into()),
            },
            PimdirError::Version { found: None },
            PimdirError::Foreign,
            PimdirError::VersionMismatch {
                user_version: 1,
                store_meta: 2,
            },
            PimdirError::HashAlgo {
                found: "md5".into(),
            },
            PimdirError::Damaged {
                object: "items".into(),
            },
            PimdirError::Exposed("pimdir.db".into()),
            PimdirError::Busy,
            PimdirError::LockReplaced("objects.lock".into()),
            PimdirError::Unsupported(PimdirRefusal {
                capability: "contacts.card.add".into(),
                source: "carddav".into(),
                detail: None,
            }),
            PimdirError::Duplicate {
                collection: "book".into(),
                link_id: "urn:uuid:c1".into(),
                existing: PimdirDuplicate::Pending(1),
            },
            PimdirError::Action(PimdirActionError::UnknownVersion(None)),
            PimdirError::IndexMissing("index.db".into()),
            PimdirError::IndexOutdated("index.db".into()),
            PimdirError::IndexVersion {
                path: "index.db".into(),
                version: Some("search-draft-10".into()),
                shape: None,
            },
            PimdirError::Fts5 { version: "3.40.0" },
            PimdirError::Query(PimdirQuery::parse("frm:alice").unwrap_err()),
        ];

        for err in failures {
            let code = ErrorCode::of_pimdir(&err).unwrap_or_else(|| panic!("no code for {err}"));
            assert_eq!(serde_json::to_value(code).unwrap(), err.code(), "{err}");
        }
    }
}
