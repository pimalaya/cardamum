//! # Pimdir client
//!
//! Wraps the reader and producer roles of [`io_pimdir`], the two Cardamum
//! takes over a store it does not own.

use std::{fmt, path::PathBuf};

use anyhow::{Error, Result, anyhow};
use io_pimdir::{
    client::{PimdirError, producer::PimdirProducer, reader::PimdirReader},
    sql::FORMAT_VERSION,
};
use pimalaya_cli::error::CodedError;

use crate::{config::PimdirConfig, error::ErrorCode};

/// The process name each staged action records (pimdir STORAGE §15.1).
///
/// Diagnostic only: it says who asked, never who applies.
const PRODUCER: &str = "cardamum";

/// Live pimdir client: a lock-free reader over the store, its blob tree
/// read through [`PimdirReader::blobs`].
///
/// A write opens a producer of its own and drops it, so this handle never
/// holds anything a sync has to wait on.
pub struct PimdirClient {
    pub(crate) reader: PimdirReader,
    /// The store root, which a producer is opened against.
    root: PathBuf,
    /// The account grouping the collections read, `None` in a store
    /// grouping none.
    pub(crate) account: Option<String>,
}

impl PimdirClient {
    /// Opens the pimdir store at the configured root to read.
    ///
    /// A reader creates and migrates nothing, the schema being the
    /// owner's to write, so a root holding no store fails here, and so
    /// does a store at another version or missing part of its schema,
    /// each with its code (see [`refusal`]). Its reads fold the pending
    /// queue over the committed rows (STORAGE §15.4), so a staged card
    /// reads back.
    ///
    /// The account is the configured one, or the store's own when it
    /// groups its collections under one account or none: see
    /// [`resolve_account`].
    pub fn new(config: PimdirConfig) -> Result<Self> {
        let PimdirConfig { root, account } = config;

        let reader = PimdirReader::open(&root)
            .map(PimdirReader::with_pending)
            .map_err(|err| refusal(err, format!("Open pimdir store `{}`", root.display())))?;
        let account = resolve_account(&reader, account)?;

        Ok(Self {
            reader,
            root,
            account,
        })
    }

    /// Opens a producer for one staging window, the enqueue-only role.
    ///
    /// Its shared lock, not the owner's exclusive one, lets several run at
    /// once. Opened per write, since the lock keeps a collector out while a
    /// body reaches the blob tree and the row pinning it is appended.
    pub(crate) fn producer(&self) -> Result<PimdirProducer> {
        let producer = PimdirProducer::open(&self.root, PRODUCER).map_err(|err| {
            let context = format!("Stage into pimdir store `{}`", self.root.display());
            refusal(err, context)
        })?;

        Ok(match self.account.clone() {
            Some(account) => producer.for_account(account),
            None => producer,
        })
    }
}

/// A pimdir failure under `context`, carrying under `--json` the code
/// io-pimdir names for it when a caller acts on it (see
/// [`ErrorCode::of_pimdir`]).
///
/// The message is io-pimdir's own, which says what the store asks of the
/// user, but for a store of another format version (see [`reinstall`]).
/// Deleting the store is never the fix, a resync restoring neither its
/// queue nor what it keeps alone.
pub(crate) fn refusal(
    err: PimdirError,
    context: impl fmt::Display + Send + Sync + 'static,
) -> Error {
    let message = match &err {
        PimdirError::Version { found } => reinstall(found.as_deref()),
        err => err.to_string(),
    };

    match ErrorCode::of_pimdir(&err) {
        Some(code) => Error::new(CodedError::new(code, message)).context(context),
        None => Error::new(err).context(context),
    }
}

/// What a store of another format version than [`FORMAT_VERSION`] asks
/// of the user, `found` being the one it records, `None` when it predates
/// the record.
///
/// A store opens under its own version alone and nothing migrates a draft
/// (pimdir STORAGE §4.2, §6), so one an earlier neverest wrote is
/// reinstalled, its queue drained first since the writes it holds live
/// there alone. A later one asks for a cardamum built on its version.
/// io-pimdir's own message says to move the store aside and sync a new
/// one, which skips the drain.
fn reinstall(found: Option<&str>) -> String {
    let found = match found {
        Some(found) => format!("is {found}"),
        None => String::from("names no format version"),
    };

    format!(
        "Pimdir store {found}, and this cardamum reads {FORMAT_VERSION} alone: \
         reinstall a store an earlier neverest wrote (run that neverest's sync \
         until nothing is queued, upgrade neverest, move the store aside, then \
         run `neverest init` and sync again), or upgrade cardamum for a store \
         a later neverest wrote"
    )
}

/// The account this client reads, configured or derived.
///
/// A store one account synced groups its collections under that name, or
/// under none in a store predating the grouping, so the common case needs
/// no configuration.
///
/// A store several accounts share is what `pimdir.account` answers, and
/// leaving it unset there errors rather than guess: one CLI view is one
/// account, and mixing their address books would write a card into one
/// that belongs to another.
fn resolve_account(reader: &PimdirReader, configured: Option<String>) -> Result<Option<String>> {
    if configured.is_some() {
        return Ok(configured);
    }

    let mut accounts: Vec<Option<String>> = reader
        .list_collections()
        .map_err(|err| anyhow!("List pimdir collections: {err}"))?
        .into_iter()
        .map(|collection| collection.account)
        .collect();
    accounts.sort();
    accounts.dedup();

    match accounts.as_slice() {
        [] | [_] => Ok(accounts.into_iter().next().flatten()),
        _ => {
            let names: Vec<&str> = accounts
                .iter()
                .map(|account| account.as_deref().unwrap_or("<ungrouped>"))
                .collect();
            Err(anyhow!(
                "The pimdir store holds several accounts ({}); name one with `pimdir.account`",
                names.join(", "),
            ))
        }
    }
}
