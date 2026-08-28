//! pimdir backend: Cardamum over a local
//! [pimdir](https://github.com/pimalaya/pimdir) store, the offline cache
//! a sync engine fills. A SQLite index plus content-addressed blobs, not
//! a live server.
//!
//! Cardamum takes two of the three roles the format defines (pimdir SPEC
//! §8) and never the third: it reads through a lock-free reader and stages
//! its writes through an enqueue-only producer, leaving the owner role,
//! which drains the queue and sweeps the store, to the sync engine.
//!
//! Reads project the store's shared items and are availability-aware: a
//! card whose body is not local lists fine from its stored summary but
//! reads as "body not fetched" rather than erroring, the cue to sync.
//! Writes are queue actions a later sync applies and propagates, and they
//! read back through the same handle that staged them.
//!
//! The vCard derivations (link id, summary) come from
//! [`io_pimdir::conventions::card`], the format's own, so a card staged
//! here links and summarizes exactly as the same card arriving through a
//! sync.

pub mod backend;
pub mod client;
