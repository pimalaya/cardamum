//! # Wizard
//!
//! Interactive account generation: one prompt orients the setup, then the
//! chosen backend asks for what it needs to authenticate.

#[cfg(feature = "carddav")]
pub mod carddav;
pub mod configure;
pub mod discover;
#[cfg(feature = "gpeople")]
pub mod gpeople;
#[cfg(feature = "jmap")]
pub mod jmap;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
pub mod local;
#[cfg(feature = "msgraph")]
pub mod msgraph;
pub mod search;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
pub mod secret;
