#[cfg(feature = "carddav")]
pub mod carddav;
pub mod discover;
#[cfg(feature = "google")]
pub mod google;
#[cfg(feature = "jmap")]
pub mod jmap;
#[cfg(feature = "msgraph")]
pub mod msgraph;
pub mod search;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "google"
))]
pub mod secret;
