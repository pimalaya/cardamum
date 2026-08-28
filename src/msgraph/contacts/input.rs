use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use io_msgraph::v1::rest::users::contacts::MsgraphContact;

/// Positional raw Graph-contact JSON source for `create` / `update`.
#[derive(Debug, Parser)]
pub struct ContactJsonArg {
    /// A path to a JSON file, raw Graph contact JSON, or `-` for stdin.
    #[arg(value_name = "JSON")]
    pub body: String,
}

impl ContactJsonArg {
    /// Reads the source and parses it into a Graph contact: `-` reads
    /// stdin, an existing file is read, otherwise the value is the JSON.
    pub fn read(self) -> Result<MsgraphContact> {
        let raw = if self.body == "-" {
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
