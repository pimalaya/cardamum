//! # REPORT commands
//!
//! One module per typed CardDAV REPORT (`addressbook-query`,
//! `addressbook-multiget`, `sync-collection`), plus a raw escape hatch
//! and the card table they share.

pub mod cli;
pub mod entries;
pub mod multiget;
pub mod query;
pub mod raw;
pub mod sync;
