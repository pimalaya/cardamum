//! # Pimdir client
//!
//! Wraps the reader and producer roles of [`io_pimdir`], the two Cardamum
//! takes over a store it does not own.

use std::path::PathBuf;

use anyhow::{Result, anyhow};
use io_pimdir::{PimdirBlobs, PimdirProducer, PimdirReader};

use crate::config::PimdirConfig;

/// The process name each staged action records (pimdir SPEC §15.1).
///
/// Diagnostic only: it says who asked, never who applies.
const PRODUCER: &str = "cardamum";

/// Live pimdir client: a lock-free reader plus a blob reader over the store.
///
/// A write opens a producer of its own and drops it, so this handle never
/// holds anything a sync has to wait on.
pub struct PimdirClient {
    pub(crate) reader: PimdirReader,
    pub(crate) blobs: PimdirBlobs,
    /// The expanded store root, which a producer is opened against.
    root: PathBuf,
    /// The account grouping the collections, `None` in a single-account store.
    pub(crate) account: Option<String>,
}

impl PimdirClient {
    /// Opens the pimdir store at the configured root to read.
    ///
    /// A reader creates nothing, the schema being the owner's to write, so a
    /// root holding no store fails here. Its reads fold the pending queue
    /// over the committed rows (SPEC §15.4), so a staged card reads back.
    pub fn new(config: PimdirConfig) -> Result<Self> {
        // NOTE: `root` carries a raw `~/…` verbatim, and opening it
        // unexpanded would look for a store at a literal `./~/…`.
        let root = shellexpand::full(&config.root.to_string_lossy())
            .map(|expanded| PathBuf::from(expanded.into_owned()))
            .unwrap_or_else(|_| config.root.clone());

        let reader = PimdirReader::open(&root)
            .map(PimdirReader::with_pending)
            .map_err(|err| anyhow!("Open pimdir store `{}`: {err}", root.display()))?;
        let blobs = reader.blobs();

        Ok(Self {
            reader,
            blobs,
            root,
            account: config.account.clone(),
        })
    }

    /// Opens a producer for one staging window, the enqueue-only role.
    ///
    /// Its shared lock, not the owner's exclusive one, lets several run at
    /// once. Opened per write, since the lock keeps a collector out of the
    /// window between a body reaching the blob tree and the row pinning it.
    pub(crate) fn producer(&self) -> Result<PimdirProducer> {
        let producer = PimdirProducer::open(&self.root, PRODUCER)
            .map_err(|err| anyhow!("Stage into pimdir store `{}`: {err}", self.root.display()))?;

        Ok(match self.account.clone() {
            Some(account) => producer.for_account(account),
            None => producer,
        })
    }
}
