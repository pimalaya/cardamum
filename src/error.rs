//! # Error codes
//!
//! A stable, machine-readable name for the failures a caller is expected
//! to act on, printed as `code` beside the message under `--json`.
//!
//! The message stays for humans and may be reworded at any time; the
//! code is the contract. A failure with no code prints as before, with
//! no `code` field. The mechanism (`CodedError`, `code_of`, the report)
//! lives in `pimalaya_cli::error`; only the vocabulary is cardamum's.

#[cfg(feature = "pimdir")]
use io_pimdir::client::PimdirError;
use schemars::JsonSchema;
use serde::Serialize;

/// The stable codes, each serialised in kebab-case.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The card is listed but its body is not local yet: a sync will
    /// bring it, so the caller waits rather than reports a loss.
    #[cfg(feature = "pimdir")]
    BodyPending,
    /// A write named, by `--if-match`, a version the card no longer has:
    /// the caller reads it again rather than overwrites a change.
    #[cfg(feature = "pimdir")]
    PreconditionFailed,
    /// The pimdir root holds no store: the sync engine owning it creates
    /// it on its first run.
    #[cfg(feature = "pimdir")]
    StoreUncreated,
    /// The pimdir store is of another format version than the one this
    /// cardamum reads, which a store opens under alone: one an earlier
    /// neverest wrote is reinstalled, a later one asks for a cardamum
    /// built on it.
    #[cfg(feature = "pimdir")]
    StoreVersion,
    /// The pimdir root holds a database another program wrote, which no
    /// pimdir tool reads or writes.
    #[cfg(feature = "pimdir")]
    StoreForeign,
    /// The pimdir store's recorded version disagrees with itself, or its
    /// schema differs from the one its version declares.
    #[cfg(feature = "pimdir")]
    StoreCorrupt,
    /// The pimdir store names its bodies by a hash this cardamum does not
    /// know.
    #[cfg(feature = "pimdir")]
    StoreHashAlgo,
    /// Another user can read or write the pimdir store's directory or
    /// database, or a link stands where a lock file belongs (pimdir
    /// STORAGE §18): nothing was opened, and the user restricts it.
    #[cfg(feature = "pimdir")]
    StoreExposed,
    /// Another writer held the pimdir store: past the wait on a write, at
    /// once on a check, which does not wait. The caller tries again.
    #[cfg(feature = "pimdir")]
    StoreBusy,
    /// A source syncing the address book does not support the write
    /// (pimdir STORAGE §15.6): nothing was queued.
    #[cfg(feature = "pimdir")]
    Unsupported,
    /// A create names a card the address book already holds, live or
    /// queued: the caller updates that card instead.
    #[cfg(feature = "pimdir")]
    ItemDuplicate,
    /// A queued action's payload is malformed (pimdir STORAGE §15.3): the
    /// store's owner parks it on its next drain.
    #[cfg(feature = "pimdir")]
    ActionMalformed,
    /// The pimdir store has no search index yet: its sync engine builds
    /// it.
    #[cfg(feature = "pimdir")]
    IndexMissing,
    /// The pimdir search index was built otherwise or from another store:
    /// its sync engine rebuilds it.
    #[cfg(feature = "pimdir")]
    IndexOutdated,
    /// The pimdir search index is of another draft or shape: its sync
    /// engine rebuilds one of an earlier draft, and leaves any other.
    #[cfg(feature = "pimdir")]
    IndexVersion,
    /// The SQLite cardamum links lacks what the pimdir search index needs
    /// (FTS5, 3.43 or later).
    #[cfg(feature = "pimdir")]
    IndexFts5Missing,
    /// The pimdir search query is malformed or names an unknown field or
    /// value (pimdir SEARCH §8.4): nothing was searched.
    #[cfg(feature = "pimdir")]
    QueryRefused,
    /// The pimdir search index could not answer an accepted query, an
    /// implementation fault.
    #[cfg(feature = "pimdir")]
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
                found: Some("storage-draft-07".into()),
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
                version: Some("search-draft-07".into()),
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
