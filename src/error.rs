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
    /// The pimdir store is at a newer version than this cardamum reads:
    /// cardamum needs an upgrade.
    #[cfg(feature = "pimdir")]
    StoreNewer,
    /// The pimdir root holds a database another program wrote, which no
    /// pimdir tool reads or writes.
    #[cfg(feature = "pimdir")]
    StoreForeign,
    /// The pimdir store lacks a table or a trigger its version declares:
    /// its sync engine, run or upgraded, reconciles it.
    #[cfg(feature = "pimdir")]
    StoreStale,
    /// The pimdir store's recorded version disagrees with itself.
    #[cfg(feature = "pimdir")]
    StoreCorrupt,
    /// The pimdir store names its bodies by a hash this cardamum does not
    /// know.
    #[cfg(feature = "pimdir")]
    StoreHashAlgo,
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
}

#[cfg(feature = "pimdir")]
impl ErrorCode {
    /// The code of a pimdir failure, read from the one io-pimdir names
    /// for it ([`PimdirError::code`]), or none for a failure no caller
    /// acts on (SQL, I/O, JSON) or no cardamum path raises.
    pub fn of_pimdir(err: &PimdirError) -> Option<Self> {
        match err.code() {
            "store-uncreated" => Some(Self::StoreUncreated),
            "store-newer" => Some(Self::StoreNewer),
            "store-foreign" => Some(Self::StoreForeign),
            "store-stale" => Some(Self::StoreStale),
            "store-corrupt" => Some(Self::StoreCorrupt),
            "store-hash-algo" => Some(Self::StoreHashAlgo),
            "store-busy" => Some(Self::StoreBusy),
            "unsupported" => Some(Self::Unsupported),
            "item-duplicate" => Some(Self::ItemDuplicate),
            _ => None,
        }
    }
}

#[cfg(all(test, feature = "pimdir"))]
mod tests {
    use io_pimdir::{
        capability::PimdirRefusal,
        client::{PimdirDuplicate, PimdirError},
    };

    use super::*;

    /// Every pimdir failure a cardamum path raises and a caller acts on
    /// prints the very code io-pimdir names for it, so a script reads one
    /// vocabulary whichever program met the store.
    #[test]
    fn a_pimdir_failure_prints_the_code_io_pimdir_names() {
        let failures = [
            PimdirError::Uncreated,
            PimdirError::Version { found: 99 },
            PimdirError::Foreign,
            PimdirError::Stale { missing: "items" },
            PimdirError::VersionMismatch {
                user_version: 1,
                store_meta: 2,
            },
            PimdirError::HashAlgo {
                found: "md5".into(),
                declared: None,
            },
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
        ];

        for err in failures {
            let code = ErrorCode::of_pimdir(&err).unwrap_or_else(|| panic!("no code for {err}"));
            assert_eq!(serde_json::to_value(code).unwrap(), err.code(), "{err}");
        }
    }
}
