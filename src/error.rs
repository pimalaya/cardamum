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

#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph"
))]
use std::fmt;

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
    /// A command would read its input off stdin, or run a composer, inside
    /// a session, where the terminal carries the commands: the caller
    /// passes it as an argument or a file path.
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
    /// A create names an address book that already exists: a vdir
    /// directory, a CardDAV collection of the home set, a Graph contact
    /// folder or a People contact group of that name. Nothing was created.
    AddressbookDuplicate,
    /// A command naming no address book found no `addressbook.default`
    /// either: the caller names one with `-k` or sets the key. Nothing was
    /// read or written.
    AddressbookDefaultMissing,
    /// The backend answered for a card cardamum could not read.
    CardUnreadable,
    /// The source given is neither a readable file nor vCard contents, or
    /// the server refused the card written as no vCard it takes (a
    /// CardDAV `valid-address-data` or `supported-address-data`
    /// precondition, or a refusal naming the vCard): nothing was written,
    /// and the caller fixes the card rather than its rights.
    CardInvalid,
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
    /// cardamum reads, which a store opens under alone: one an earlier sync
    /// engine wrote is drained by it, then set aside, a later one asks for
    /// a later cardamum.
    StoreVersion,
    /// The pimdir root holds a database another program wrote, which no
    /// pimdir tool reads or writes.
    StoreForeign,
    /// The pimdir store's recorded version disagrees with itself, or its
    /// schema differs from the one its version declares.
    StoreCorrupt,
    /// The pimdir store holds no schema yet, and the verb creates none:
    /// its sync engine creates it.
    StoreNotCurrent,
    /// The pimdir store names its bodies by a hash this cardamum does not
    /// know.
    StoreHashAlgo,
    /// The pimdir store's sync state was written by another version of
    /// its sync engine.
    SyncVersion,
    /// Another user can read or write the pimdir store's directory or
    /// database, or a link stands where a lock file belongs (pimdir
    /// STORAGE §18): nothing was opened, and the user restricts it.
    StoreExposed,
    /// The pimdir store is not the one that answered the queue row's
    /// handle, a copy restored since: nothing was done with the row.
    StoreReset,
    /// Another writer held the pimdir store: past the wait on a write, at
    /// once on a check, which does not wait. The caller tries again.
    StoreBusy,
    /// A sync owns the pimdir store: retry once it ends.
    StoreOwned,
    /// A producer is staging a body in the pimdir store: retry.
    StoreStaging,
    /// A pimdir store verb is between two of its chunks: retry.
    StoreInFlight,
    /// The store's staging lock file was replaced while a write held it
    /// (pimdir STORAGE §8): nothing was queued, and the caller tries
    /// again once no other process is setting the store up.
    LockReplaced,
    /// A source syncing the address book does not support the write
    /// (pimdir STORAGE §15.6): nothing was queued.
    Unsupported,
    /// A create names a card the address book already holds, live or
    /// queued, on pimdir, or a `UID` a CardDAV address book holds: the
    /// caller updates that card instead.
    ItemDuplicate,
    /// The pimdir store cannot hold the flag: empty, or holding a space, a
    /// control or a non-ASCII character. Nothing was staged.
    FlagInvalid,
    /// A queued action's payload is malformed (pimdir STORAGE §15.3): the
    /// store's owner parks it on its next drain.
    ActionMalformed,
    /// The pimdir conflict holds a body nobody has read: fetch it first.
    ConflictUnread,
    /// No source of the pimdir store can perform the intent.
    NoPerformer,
    /// Several sources of the pimdir store can perform the intent and
    /// none is chosen.
    AmbiguousPerformer,
    /// The pimdir store has no search index yet: its sync engine builds
    /// it.
    IndexMissing,
    /// The pimdir search index was built otherwise or from another store:
    /// its sync engine rebuilds it.
    IndexOutdated,
    /// The pimdir search index is of another format version or shape: its
    /// sync engine rebuilds one of an earlier version, and leaves any other.
    IndexVersion,
    /// The zone the pimdir search index reads floating times in is not an
    /// IANA zone.
    IndexZoneUnknown,
    /// The SQLite cardamum links lacks what the pimdir search index needs
    /// (FTS5, 3.43 or later).
    IndexFts5Missing,
    /// The pimdir search query is malformed or names an unknown field or
    /// value (pimdir SEARCH §8.4): nothing was searched.
    QueryRefused,
    /// The search query holds a term only a pimdir store answers (a tag,
    /// a flag, a change stamp), on a backend holding none: nothing was
    /// searched.
    QueryUnsupported,
    /// The pimdir search index could not answer an accepted query, an
    /// implementation fault.
    QueryUnanswerable,
    /// The pimdir tag names no flag, or adds and removes one flag: nothing
    /// was staged.
    TagRefused,
    /// The pimdir store holds several accounts and `pimdir.account` names
    /// none: nothing was read.
    StoreAccountAmbiguous,
    /// The pimdir store holds no collection of the account
    /// `pimdir.account` names: nothing was read.
    StoreAccountUnknown,
    /// No pimdir queue row of the account has the id given: nothing was
    /// cancelled.
    RowNotFound,
    /// The command asks to confirm, and no terminal can answer: pass
    /// `--yes`. Nothing was done.
    ConfirmationRequired,
    /// The page token is not one the command printed: nothing was listed.
    PageTokenInvalid,
}

/// The pimdir codes cardamum relays, by io-pimdir's spelling, which
/// serialises each: Himalaya's set, so one store failure reads the same
/// code in every program of the family.
#[cfg(feature = "pimdir")]
const PIMDIR_CODES: &[(&str, ErrorCode)] = &[
    ("store-version", ErrorCode::StoreVersion),
    ("store-uncreated", ErrorCode::StoreUncreated),
    ("store-foreign", ErrorCode::StoreForeign),
    ("store-corrupt", ErrorCode::StoreCorrupt),
    ("store-not-current", ErrorCode::StoreNotCurrent),
    ("store-hash-algo", ErrorCode::StoreHashAlgo),
    ("sync-version", ErrorCode::SyncVersion),
    ("store-busy", ErrorCode::StoreBusy),
    ("store-owned", ErrorCode::StoreOwned),
    ("store-staging", ErrorCode::StoreStaging),
    ("store-in-flight", ErrorCode::StoreInFlight),
    ("store-exposed", ErrorCode::StoreExposed),
    ("store-reset", ErrorCode::StoreReset),
    ("lock-replaced", ErrorCode::LockReplaced),
    ("item-duplicate", ErrorCode::ItemDuplicate),
    ("flag-invalid", ErrorCode::FlagInvalid),
    ("action-malformed", ErrorCode::ActionMalformed),
    ("conflict-unread", ErrorCode::ConflictUnread),
    ("unsupported", ErrorCode::Unsupported),
    ("no-performer", ErrorCode::NoPerformer),
    ("ambiguous-performer", ErrorCode::AmbiguousPerformer),
    ("index-missing", ErrorCode::IndexMissing),
    ("index-outdated", ErrorCode::IndexOutdated),
    ("index-version", ErrorCode::IndexVersion),
    ("index-zone-unknown", ErrorCode::IndexZoneUnknown),
    ("index-fts5-missing", ErrorCode::IndexFts5Missing),
    ("query-refused", ErrorCode::QueryRefused),
    ("query-unanswerable", ErrorCode::QueryUnanswerable),
    ("tag-refused", ErrorCode::TagRefused),
];

/// The refusal of a page token the command did not print, under
/// `page-token-invalid`.
pub fn page_token_invalid(token: &str) -> Error {
    let message = format!("Invalid page token {token}");
    pimalaya_cli::error::CodedError::new(ErrorCode::PageTokenInvalid, message).into()
}

#[cfg(feature = "pimdir")]
impl ErrorCode {
    /// The code of a pimdir failure, read from the one io-pimdir names
    /// for it ([`PimdirError::code`]), or none for a failure no caller
    /// acts on (SQL, I/O, JSON) or no program of the family relays.
    pub fn of_pimdir(err: &PimdirError) -> Option<Self> {
        let code = err.code();
        PIMDIR_CODES
            .iter()
            .find(|(pimdir, _)| *pimdir == code)
            .map(|(_, ours)| *ours)
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
        #[cfg(any(
            feature = "carddav",
            feature = "gpeople",
            feature = "jmap",
            feature = "msgraph"
        ))]
        coded_cause_code,
        account_code,
        #[cfg(feature = "jmap")]
        crate::jmap::error::refusal_code,
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

/// A typed error a code was put on, kept whole in the chain: a protocol
/// command still prints what the server answered
/// ([`crate::shared::refusal`]), which a [`with_code`] rebuilding the
/// chain from its messages would lose.
#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph"
))]
#[derive(Debug)]
pub struct CodedCause {
    code: ErrorCode,
    cause: Box<dyn std::error::Error + Send + Sync + 'static>,
}

#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph"
))]
impl CodedCause {
    /// The typed error, as the backend returned it.
    pub fn cause(&self) -> &(dyn std::error::Error + 'static) {
        &*self.cause
    }
}

#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph"
))]
impl fmt::Display for CodedCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.cause, f)
    }
}

#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph"
))]
impl std::error::Error for CodedCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.source()
    }
}

/// `cause` carrying `code`, the typed error kept whole.
#[cfg(all(
    any(
        feature = "carddav",
        feature = "gpeople",
        feature = "jmap",
        feature = "msgraph"
    ),
    any(feature = "carddav", feature = "gpeople", feature = "msgraph", test)
))]
pub fn coded_cause(
    cause: impl std::error::Error + Send + Sync + 'static,
    code: ErrorCode,
) -> Error {
    Error::new(CodedCause {
        code,
        cause: Box::new(cause),
    })
}

/// The code a [`CodedCause`] carries.
#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph"
))]
fn coded_cause_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    cause.downcast_ref::<CodedCause>().map(|coded| coded.code)
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

/// Whether `version`, the one a card read reports, is the one `if_match`
/// names, surrounding double quotes ignored.
#[cfg(any(feature = "jmap", feature = "msgraph"))]
pub fn version_matches(version: Option<&str>, if_match: &str) -> bool {
    let unquote = |etag: &str| etag.trim_matches('"').to_owned();
    version.map(unquote) == Some(unquote(if_match))
}

/// The refusal of a write `--if-match` guards, the card `card_id` having
/// changed on `server` since it was read: `precondition-failed`.
#[cfg(any(feature = "jmap", feature = "msgraph"))]
pub fn card_changed(card_id: &str, server: &str) -> Error {
    let message = format!("Card `{card_id}` changed on {server} since it was read");
    pimalaya_cli::error::CodedError::new(ErrorCode::PreconditionFailed, message).into()
}

/// The failure of the card `card_id` the address book `addressbook_id`
/// does not hold, `cause` what the backend answered: cardamum's sentence,
/// as Himalaya words a message not found in its mailbox, the backend's
/// text in `sources`.
#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph",
    feature = "vdir"
))]
pub fn card_not_found(addressbook_id: &str, card_id: &str, cause: Error) -> Error {
    let message = format!("Card `{card_id}` not found in address book `{addressbook_id}`");
    with_code(cause, |_| Some(ErrorCode::CardNotFound)).context(message)
}

/// The failure to read the card `card_id` the backend holds, `cause` why:
/// cardamum's sentence naming it, coded as `cause` is or
/// `card-unreadable`, as a listing codes its entry.
#[cfg(any(feature = "pimdir", feature = "vdir"))]
pub fn card_unreadable(addressbook_id: &str, card_id: &str, cause: Error) -> Error {
    let message = format!("Cannot read card `{card_id}` in address book `{addressbook_id}`");
    with_code(cause, |_| Some(ErrorCode::CardUnreadable)).context(message)
}

/// The failure of the address book `id` the backend does not hold, `cause`
/// what it answered: cardamum's sentence naming it, the backend's text in
/// `sources`.
#[cfg(any(
    feature = "carddav",
    feature = "gpeople",
    feature = "jmap",
    feature = "msgraph"
))]
pub fn addressbook_not_found(id: &str, cause: Error) -> Error {
    with_code(cause, |_| Some(ErrorCode::AddressbookNotFound))
        .context(format!("Addressbook `{id}` not found"))
}

/// The failure of a create naming the address book `name` the backend
/// already holds, `cause` what it answered: cardamum's sentence naming it,
/// the backend's text in `sources`.
#[cfg(any(feature = "carddav", feature = "gpeople", feature = "msgraph"))]
pub fn addressbook_duplicate(name: &str, cause: Error) -> Error {
    with_code(cause, |_| Some(ErrorCode::AddressbookDuplicate)).context(format!(
        "Cannot create address book `{name}`: it exists already"
    ))
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
        client::{PimdirDuplicate, PimdirError, PimdirNotCurrent},
        codec::PimdirActionError,
        search::query::PimdirQuery,
    };

    use super::*;

    /// Every relayed code keeps io-pimdir's spelling.
    #[test]
    fn a_relayed_pimdir_code_keeps_io_pimdirs_spelling() {
        for (pimdir, ours) in PIMDIR_CODES {
            assert_eq!(serde_json::to_value(ours).unwrap(), *pimdir);
        }
    }

    /// Every pimdir failure a caller acts on prints the very code io-pimdir
    /// names for it, so a script reads one vocabulary whichever program met
    /// the store, Himalaya's set included.
    #[test]
    fn a_pimdir_failure_prints_the_code_io_pimdir_names() {
        let failures = [
            PimdirError::Uncreated,
            PimdirError::Version {
                found: Some("storage-draft-11".into()),
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
                version: Some("search-draft-11".into()),
                shape: None,
            },
            PimdirError::Fts5 { version: "3.40.0" },
            PimdirError::Query(PimdirQuery::parse("frm:alice").unwrap_err()),
            PimdirError::NotCurrent(PimdirNotCurrent::Version { found: 0 }),
            PimdirError::SyncVersion {
                found: "sync-draft-08".into(),
            },
            PimdirError::Owned("pimdir.lock".into()),
            PimdirError::Staging("objects.lock".into()),
            PimdirError::InFlight("pimdir.db".into()),
            PimdirError::Flag(" ".into()),
            PimdirError::NoPerformer {
                capability: "calendar-reply".into(),
            },
            PimdirError::Ambiguous {
                capability: "calendar-reply".into(),
                candidates: vec!["a".into(), "b".into()],
            },
            PimdirError::IndexZone("Mars/Olympus".into()),
        ];

        for err in failures {
            let code = ErrorCode::of_pimdir(&err).unwrap_or_else(|| panic!("no code for {err}"));
            assert_eq!(serde_json::to_value(code).unwrap(), err.code(), "{err}");
        }
    }
}
