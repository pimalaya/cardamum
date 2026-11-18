//! # Contact JSON input
//!
//! The raw Graph contact body shared by the create and update commands.

use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;

/// Positional raw Graph-contact JSON source for `create` / `update`.
#[derive(Debug, Parser)]
pub struct ContactJsonArg {
    /// A path to a JSON file, raw Graph contact JSON, or `-` for stdin.
    #[arg(value_name = "JSON")]
    pub body: String,
}

impl ContactJsonArg {
    /// Reads the source as raw JSON, sent to Graph as written: a key
    /// io-msgraph's contact model lacks reaches the server.
    ///
    /// A `-` reads stdin, an existing path is read as a file, and anything
    /// else is the JSON itself.
    pub fn read(self) -> Result<serde_json::Value> {
        let raw = if self.body == "-" {
            pimalaya_cli::session::check_stdin(crate::error::ErrorCode::StdinTaken)?;
            let mut buf = String::new();
            stdin()
                .read_to_string(&mut buf)
                .context("Read contact JSON from stdin error")?;
            buf
        } else {
            let path = PathBuf::from(&self.body);
            if path.is_file() {
                fs::read_to_string(&path)
                    .with_context(|| format!("Read contact JSON from `{}` error", path.display()))?
            } else {
                self.body
            }
        };

        serde_json::from_str(&raw).context("Parse Graph contact JSON error")
    }
}
