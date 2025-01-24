#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod account;
pub mod addressbook;
pub mod card;
#[cfg(feature = "_carddav")]
pub mod carddav;
pub mod cli;
mod client;
pub mod completion;
pub mod config;
pub mod manual;
pub mod table;
#[cfg(feature = "_vdir")]
pub mod vdir;
#[cfg(feature = "wizard")]
pub mod wizard;

#[doc(inline)]
pub use client::Client;
