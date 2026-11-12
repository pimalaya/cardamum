---
cairn: change
id: machine-output-contract
status: landed
created: 2026-10-27
---

# Adopt the shared machine output contract

## Why

pimalaya-cli now owns the machine contract every binary prints (cli `machine-output-contract`, commit `55b6693`): the coded-error layer Himalaya introduced and cardamum copied (json-error-code), a schema for the failure object, schemas stamped with the contract version, piped `--json` delimited by newlines, and a check that every payload is a JSON object (what an MCP client takes as structured output).

## What

- src/error.rs keeps only the `ErrorCode` vocabulary; `CodedError` and `code_of` come from `pimalaya_cli::error`, and main.rs evaluates through `ErrorReport::<ErrorCode>::eval_coded`.
- src/json_schema.rs returns a `JsonSchemaRegistry` (bin `cardamum`, the crate version), registers `cardamum-error`, which the key-names-a-command test exempts, and tests that no payload root is anything but an object.
- 25 roots were not objects, in three groups:
  - The wire objects of the `gpeople`, `jmap` and `msgraph` families (a person, a contact group, an address book, a contact card, the session, a contact folder, a contact, the profile) were described as any JSON value. They are described as a JSON object of any shape: what they always were, still claiming no shape we do not own. Output unchanged.
  - The abandoned arm of `card create` and `card update` serialized as `null`. It becomes an empty struct variant, printing `{}`; it is reachable through `-i` alone, which `--json` refuses, so no JSON a caller can get moves.
  - The `request` passthroughs of `gpeople`, `jmap` and `msgraph` printed the response verbatim, which may be `null` (a 204) or an array. They now print `{"response": <the response>}`. Breaking for `request --json`; the text output is unchanged.
- pimalaya-cli is pinned at `55b6693` until its 0.3.0 release.
