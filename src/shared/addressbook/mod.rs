//! # Addressbook
//!
//! The shared addressbook types and the commands operating on them.

mod types;

pub mod cli;
pub mod create;
pub mod delete;
pub mod list;
pub mod update;

#[doc(inline)]
pub use types::*;
