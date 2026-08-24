//! pimdir backend: Cardamum over a local
//! [pimdir](https://github.com/pimalaya/pimdir) store, the offline cache
//! a sync engine fills. A SQLite index plus content-addressed blobs, not
//! a live server.
//!
//! Reads project the store's shared items and are availability-aware: a
//! card whose body is not local lists fine from its stored summary but
//! reads as "body not fetched" rather than erroring, the cue to sync.
//! Writes are staged io-replica mutations a later sync propagates,
//! attributed to the configured
//! [`source`](crate::config::PimdirConfig::source), which must match the sync
//! source for the change to reach the servers.
//!
//! The [`card`] module holds the vCard derivations (link id, summary, sort key)
//! and must stay byte-compatible with Neverest's `text/vcard` kind, since the
//! two write into the same store.

pub mod backend;
pub mod card;
pub mod client;
