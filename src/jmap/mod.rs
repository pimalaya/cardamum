//! # JMAP
//!
//! The JMAP arm (RFC 8620) and its contacts extension (RFC 9610): the
//! shared-API adapter, the protocol-specific command tree, and the
//! projection between ContactCards and vCard documents.

pub mod address_book;
pub mod backend;
pub mod cli;
pub mod client;
pub mod contact_card;
pub mod error;
pub mod input;
pub mod project;
pub mod render;
pub mod request;
pub mod session;
