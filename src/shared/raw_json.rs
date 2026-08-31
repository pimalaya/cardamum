//! # Raw JSON
//!
//! The output type of the protocol `request` passthroughs, printing the
//! payload a backend answered as is.

use core::fmt;

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

/// A raw JSON payload: pretty on stdout, verbatim with `--json`.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct RawJsonOutput(pub Value);

impl fmt::Display for RawJsonOutput {
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
        assert_eq!(RawJsonOutput(Value::Null).to_string(), "");
        assert_eq!(
            RawJsonOutput(json!({"id": "x"})).to_string(),
            "{\n  \"id\": \"x\"\n}"
        );
    }
}
