use core::fmt;

use serde::Serialize;
use serde_json::Value;

/// A raw JSON payload from a protocol `request` passthrough: pretty on
/// stdout, verbatim with `--json`.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct RawJson(pub Value);

impl fmt::Display for RawJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pretty = serde_json::to_string_pretty(&self.0).map_err(|_| fmt::Error)?;
        write!(f, "{pretty}")
    }
}
