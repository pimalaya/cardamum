use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use serde_json::{Map, Value};

/// Positional raw JSContact JSON source for `contact-card create` /
/// `update`.
#[derive(Debug, Parser)]
pub struct JsonArg {
    /// A path to a JSON file, raw JSContact JSON, or `-` for stdin.
    #[arg(value_name = "JSON")]
    pub body: String,
}

impl JsonArg {
    /// Reads the source into a JSON object: `-` reads stdin, an existing
    /// file is read, otherwise the value is the JSON itself.
    pub fn read(self) -> Result<Map<String, Value>> {
        let raw = if self.body == "-" {
            let mut buf = String::new();
            stdin()
                .read_to_string(&mut buf)
                .context("Read JSContact from stdin error")?;
            buf
        } else {
            let path = PathBuf::from(&self.body);
            if path.is_file() {
                fs::read_to_string(&path)
                    .with_context(|| format!("Read JSContact from `{}` error", path.display()))?
            } else {
                self.body
            }
        };

        match serde_json::from_str(&raw).context("Parse JSContact JSON error")? {
            Value::Object(map) => Ok(map),
            _ => anyhow::bail!("JSContact must be a JSON object"),
        }
    }
}
