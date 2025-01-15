//! # Sans I/O
//!
//! This module contains the state machine [`Flow`] and [`Io`]
//! definitions, as well as commonly-used flows definition like
//! [`ReadEntryFlow`], [`WriteEntryFlow`] and [`DeleteEntryFlow`].

#[path = "flow-contact-list.rs"]
mod flow_contact_list;

#[doc(inline)]
pub use self::flow_contact_list::*;
