//! # Vdir
//!
//! Vdir-specific API over a local directory tree: a collection is a
//! directory under the account's home directory, an item a vCard or
//! iCalendar file inside it.

pub mod backend;
pub mod cli;
pub mod client;
pub mod create;
pub mod delete;
pub mod item;
pub mod list;
pub mod rename;
