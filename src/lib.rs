#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod account;
pub mod addressbook;
pub mod card;
#[cfg(feature = "carddav")]
pub mod carddav;
pub mod cli;
mod client;
pub mod config;
pub mod table;
#[cfg(feature = "vdir")]
pub mod vdir;
// #[cfg(feature = "wizard")]
// pub mod wizard;
