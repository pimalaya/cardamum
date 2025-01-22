#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod account;
pub mod addressbook;
pub mod card;
#[cfg(feature = "_carddav")]
pub mod carddav;
pub mod cli;
pub mod completion;
pub mod config;
pub mod manual;
pub mod table;
