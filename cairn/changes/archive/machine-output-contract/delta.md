---
cairn: delta
change: machine-output-contract
---

## ADDED Requirements

### Requirement: The JSON contract is described and versioned
`json-schema` SHALL describe the failure object under the key `cardamum-error`, its `code` typed by the code vocabulary. Every generated schema SHALL carry an `$id` naming the contract version: the major version of cardamum, or `0.<minor>` before 1.0. Every registered payload SHALL be a JSON object, checked by a test: a command whose shapes are one untagged type answers an empty object for a shape carrying nothing (an abandoned `card create -i`), never `null`. Piped `--json` output SHALL print one value per line.

## MODIFIED Requirements

### Requirement: Specific commands are raw-faithful
A specific command SHALL surface the native ids and metadata the shared layer normalizes away (ETags, CTags, sync tokens, hrefs, the Graph `changeKey`, the People `etag` and `resourceName`), and SHALL make the incremental-sync primitive its headline (`report sync`, `changes`, `delta`). Default human output is a friendly table; `--json` SHALL emit the raw native payload, and create and update SHALL accept the raw native representation. Each backend keeps a raw escape hatch: a raw XML body for CardDAV, raw JSON in and out plus a generic request passthrough for the object backends, whose `--json` prints the response verbatim under `response` (`null` when the server sent no body), so the payload is an object whatever the server answered. Precondition control is offered only where the protocol has it (CardDAV `--if-match` and `--if-none-match`).

### Requirement: The JSON output of every data command has a schema
Every output type SHALL be registered in [json_schema.rs](../../../src/json_schema.rs) under its CLI invocation path, hyphen-joined and prefixed `cardamum-`, and the `json-schema` command (aliased `json-schemas`) SHALL print one or write one file per entry. A registered key naming no command is a test failure, so the registry cannot drift from the tree; `cardamum-error`, the failure object every command prints, is the one key naming none.

A wire object the protocol crate exposes without a `JsonSchema` of its own SHALL be described as a JSON object of any shape rather than left out: the payload is the provider's to define, and claiming a shape we do not own would be worse than claiming none.
