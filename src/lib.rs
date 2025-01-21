#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod account;
pub mod addressbook;
// pub mod card;
#[cfg(any(
    feature = "carddav",
    feature = "carddav-native-tls",
    feature = "carddav-rustls",
))]
pub mod carddav;
pub mod cli;
pub mod completion;
pub mod config;
pub mod manual;
pub mod table;
