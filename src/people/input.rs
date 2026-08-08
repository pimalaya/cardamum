use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use io_people::v1::rest::people::PeoplePersonField;
use serde_json::Value;

/// Positional raw People-person JSON source for `connection create` /
/// `update`.
#[derive(Debug, Parser)]
pub struct PersonJsonArg {
    /// A path to a JSON file, raw People person JSON, or `-` for stdin.
    #[arg(value_name = "JSON")]
    pub body: String,
}

impl PersonJsonArg {
    /// Reads the source into a raw JSON value: `-` reads stdin, an
    /// existing file is read, otherwise the value is the JSON itself.
    pub fn read(self) -> Result<Value> {
        let raw = if self.body == "-" {
            let mut buf = String::new();
            stdin()
                .read_to_string(&mut buf)
                .context("Read person JSON from stdin error")?;
            buf
        } else {
            let path = PathBuf::from(&self.body);
            if path.is_file() {
                fs::read_to_string(&path)
                    .with_context(|| format!("Read person JSON from `{}` error", path.display()))?
            } else {
                self.body
            }
        };

        serde_json::from_str(&raw).context("Parse People person JSON error")
    }
}

/// The People `updatePersonFields` mask derived from the top-level keys
/// of the edit JSON: each key that names a person field (`names`,
/// `emailAddresses`, …) becomes one mask entry; unknown keys (`etag`,
/// `resourceName`) are ignored.
pub fn update_fields_from_json(value: &Value) -> Vec<PeoplePersonField> {
    value
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(key, _)| serde_json::from_value(Value::String(key.clone())).ok())
        .collect()
}
