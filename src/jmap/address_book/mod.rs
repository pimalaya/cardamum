//! # AddressBook
//!
//! The `address-book` command tree, one module per JMAP `AddressBook`
//! method (RFC 9610 §2).

pub mod changes;
pub mod cli;
pub mod create;
pub mod destroy;
pub mod get;
pub mod update;
