#[cfg(feature = "carddav")]
pub mod carddav;
pub mod discover;
#[cfg(feature = "jmap")]
pub mod jmap;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
pub mod local;
#[cfg(feature = "msgraph")]
pub mod msgraph;
#[cfg(feature = "people")]
pub mod people;
pub mod search;
#[cfg(any(
    feature = "carddav",
    feature = "jmap",
    feature = "msgraph",
    feature = "people"
))]
pub mod secret;
