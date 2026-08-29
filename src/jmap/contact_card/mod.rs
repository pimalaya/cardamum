//! # ContactCard
//!
//! The `contact-card` command tree, one module per JMAP `ContactCard`
//! method (RFC 9610 §3).

pub mod changes;
pub mod cli;
pub mod copy;
pub mod create;
pub mod destroy;
pub mod get;
pub mod query;
pub mod update;
