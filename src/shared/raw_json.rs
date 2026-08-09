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
        // NOTE: a 204 carries no body, which parses as null. Printing
        // "null" would read as a value the server sent; it sent nothing.
        if self.0.is_null() {
            return Ok(());
        }

        let pretty = serde_json::to_string_pretty(&self.0).map_err(|_| fmt::Error)?;
        write!(f, "{pretty}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_empty_response_prints_nothing() {
        assert_eq!(RawJson(Value::Null).to_string(), "");
        assert_eq!(
            RawJson(json!({"id": "x"})).to_string(),
            "{\n  \"id\": \"x\"\n}"
        );
    }
}
