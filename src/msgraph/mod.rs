//! # Microsoft Graph
//!
//! The Graph arm of the shared-API client ([`backend`]), the Graph-specific
//! command tree ([`cli`]) and the contact to vCard projection they both
//! rest on ([`project`]).

pub mod backend;
pub mod cli;
pub mod client;
pub mod contact_folders;
pub mod contacts;
pub mod profile;
pub mod project;
pub mod request;
