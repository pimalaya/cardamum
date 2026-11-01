//! # Pimdir
//!
//! Cardamum over a local [pimdir](https://github.com/pimalaya/pimdir)
//! store, the offline cache a sync engine fills: a SQLite index plus
//! content-addressed blobs, not a live server.
//!
//! Cardamum takes two of the three roles the format defines (pimdir
//! STORAGE §8) and never the third: it reads through a lock-free reader
//! and stages its writes through an enqueue-only producer, leaving the
//! owner role, which drains the queue, to the sync engine.
//!
//! Reads are availability-aware: a card whose body is not local lists
//! fine from its stored summary but reads as "body not fetched" rather
//! than erroring, the cue to sync. Writes are queue actions a later sync
//! applies: a staged update or removal reads back at once, a staged create
//! lists once the sync applies it, having no public id before. A staged
//! create is what `pimdir status` counts, and a write the owner parked
//! rather than applied is what it lists.
//!
//! The vCard derivations (link id, summary, sort key) are
//! [`io_pimdir::summary`]'s, the format's own: the owner derives them
//! from the body a staged action carries, so a card staged here links and
//! summarizes exactly as one arriving through a sync.

pub mod backend;
pub mod cli;
pub mod client;
pub mod status;
