#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

#[path = "sans-io/mod.rs"]
pub mod sans_io;
pub mod serde;
pub mod std;
pub mod tcp;
