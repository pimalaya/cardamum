//! # Sans I/O
//!
//! This module contains the state machine [`Flow`] and [`Io`]
//! definitions, as well as commonly-used flows definition like
//! [`ReadEntryFlow`], [`WriteEntryFlow`] and [`DeleteEntryFlow`].

mod flow;
#[path = "flow-contact-list.rs"]
mod flow_contact_list;
mod io;

#[doc(inline)]
pub use self::{flow::*, flow_contact_list::*, io::*};
