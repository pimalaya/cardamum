---
cairn: log
change: machine-output-contract
landed: 2026-10-27
---

# Adopted the shared machine output contract

The coded-error mechanism moved to pimalaya-cli (`55b6693`, pinned until 0.3.0); src/error.rs keeps only the `ErrorCode` vocabulary, and main.rs evaluates through `ErrorReport::<ErrorCode>::eval_coded`. The JSON error object prints exactly as before.

`json-schema` gained `cardamum-error` (exempt from the key-names-a-command test), and each schema now carries `$id` = `https://pimalaya.org/schemas/cardamum/v0.3/<key>.json`. A test asserts every payload root is an object; 25 were not:

- 20 wire objects of `gpeople`, `jmap` and `msgraph` were schema'd as any value (`#[schemars(with = "serde_json::Value")]`); they are now `serde_json::Map<String, Value>`, an object of any shape. Output unchanged.
- `card create` and `card update`: the untagged abandoned arm printed `null`; it is now an empty struct variant printing `{}`, reachable only through `-i`, which `--json` refuses.
- `gpeople request`, `jmap request` and `msgraph request`: `RawJsonOutput` was transparent, so a 204 printed `null`. It now prints `{"response": …}`. Breaking for `request --json`; text output unchanged.

Piped `--json` is now one value per line, from the pimalaya-cli printer.

Verified: 67 binary tests pass with all features, clippy clean on all features, `--no-default-features` builds (27 pre-existing warnings, one fewer than before).

Spec updated: commands (ADDED: The JSON contract is described and versioned; MODIFIED: Specific commands are raw-faithful, The JSON output of every data command has a schema).
