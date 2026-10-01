//! # Google People
//!
//! Google People support: the protocol-specific command tree and the
//! shared-API backend. The person to vCard projection they rest on is
//! io-gpeople's `vcard` feature.

pub mod backend;
pub mod cli;
pub mod client;
pub mod connection;
pub mod contact_group;
pub mod input;
pub mod other_contact;
pub mod profile;
pub mod render;
pub mod request;
