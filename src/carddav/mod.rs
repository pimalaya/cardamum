//! # CardDAV
//!
//! The CardDAV side of cardamum: the client, its arm of the shared-API
//! backend, and a CLI command per WebDAV method (RFC 4918, 5689, 6352,
//! 6578).

pub mod backend;
pub mod cli;
pub mod client;
pub mod delete;
pub mod discover;
pub mod get;
pub mod mkcol;
pub mod propfind;
pub mod proppatch;
pub mod put;
pub mod report;
