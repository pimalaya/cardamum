//! # Raw JMAP gets
//!
//! A `Foo/get` sent through io-jmap's raw send, its objects kept as the
//! server sent them, a property io-jmap does not model included, as
//! Himalaya's `jmap` gets read theirs.

use anyhow::{Context, Result};
use io_jmap::{
    rfc8620::{JMAP_CORE_CAPABILITY, error::JmapMethodError, request::JmapRequest},
    rfc9610::JMAP_CONTACTS_CAPABILITY,
};
use serde_json::{Map, Value, json};

use crate::jmap::client::JmapClient;

/// What a `Foo/get` answered: its objects as sent, and its state.
pub struct JmapRawList {
    /// The objects of `list`, as the server sent them.
    pub list: Vec<Value>,
    /// The `state` of the answer.
    pub state: String,
}

/// Sends `<object>/get` of `ids` (every object when `None`) under the
/// contacts capability; a method answering `error` fails with it.
pub fn get(client: &mut JmapClient, object: &str, ids: Option<Vec<String>>) -> Result<JmapRawList> {
    let session = client.session().context("No JMAP session")?;
    let account_id = session.account_for(JMAP_CONTACTS_CAPABILITY)?.to_owned();
    let args = json!({"accountId": account_id, "ids": ids});
    let request = JmapRequest {
        using: vec![
            JMAP_CORE_CAPABILITY.to_string(),
            JMAP_CONTACTS_CAPABILITY.to_string(),
        ],
        method_calls: vec![(format!("{object}/get"), args, String::from("c0"))],
        created_ids: None,
    };
    let response = client.send_raw(request)?;

    let Some((name, args, _)) = response.method_responses.into_iter().next() else {
        anyhow::bail!("The JMAP server answered no {object}/get");
    };

    if name == "error" {
        let err: JmapMethodError = serde_json::from_str(args.get())
            .with_context(|| format!("Cannot read the error of {object}/get"))?;
        return Err(err.into());
    }

    let mut get: Map<String, Value> = serde_json::from_str(args.get())
        .with_context(|| format!("Cannot read the answer to {object}/get"))?;
    let list = match get.remove("list") {
        Some(Value::Array(list)) => list,
        _ => Vec::new(),
    };
    let state = match get.remove("state") {
        Some(Value::String(state)) => state,
        _ => String::new(),
    };

    Ok(JmapRawList { list, state })
}
