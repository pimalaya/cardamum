//! # Microsoft Graph
//!
//! The Graph arm of the shared-API client ([`backend`]) and the
//! Graph-specific command tree ([`cli`]). The contact to vCard projection
//! they rest on is io-msgraph's `vcard` feature.

pub mod backend;
pub mod cli;
pub mod client;
pub mod contact_folders;
pub mod contacts;
pub mod profile;
pub mod request;
