//! # Card
//!
//! The shared card types and the commands operating on them.

mod types;

pub mod add;
pub mod build;
pub mod cli;
pub mod composer;
pub mod delete;
pub mod fields;
pub mod list;
pub mod project;
pub mod read;
#[cfg(any(
    feature = "vdir",
    feature = "pimdir",
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
pub mod search;
pub mod update;
pub mod vcard;

#[doc(inline)]
pub use types::*;
