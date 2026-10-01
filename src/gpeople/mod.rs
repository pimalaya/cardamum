//! # Google People
//!
//! Google People support: the protocol-specific command tree, the
//! shared-API backend, and the person to vCard projection they share.

pub mod backend;
pub mod cli;
pub mod client;
pub mod connection;
pub mod contact_group;
pub mod input;
pub mod other_contact;
pub mod profile;
pub mod project;
pub mod render;
pub mod request;
