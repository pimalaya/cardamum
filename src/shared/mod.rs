//! # Shared API
//!
//! Protocol-agnostic addressbook and card commands, run against whichever
//! backend the active account defines.

pub mod addressbook;
pub mod arg;
pub mod card;
pub mod client;
pub mod note;
#[cfg(any(feature = "jmap", feature = "msgraph", feature = "gpeople"))]
pub mod raw_json;
pub mod table;
pub mod uuid;
