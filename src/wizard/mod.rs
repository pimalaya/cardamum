//! # Wizard
//!
//! Interactive account generation: one prompt orients the setup, discovery
//! picks the best way to the contacts, the review shows it, then the
//! sign-in is typed once and tested.

pub mod configure;
pub mod discover;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
pub mod local;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
pub mod remote;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
pub mod review;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
pub mod search;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
pub mod secret;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
pub mod server;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "gpeople"
))]
pub mod sign_in;
