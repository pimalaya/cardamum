---
cairn: change
id: output-schemas-and-path-expansion
status: landed
created: 2026-08-29
---

# Delta

## ADDED Requirements

### Requirement: A DAV server may ask for no credentials
`CarddavAuthConfig` SHALL carry a credential-less variant alongside Basic and Bearer, spelled `auth = "none"`, which emits no `Authorization` header. The wizard SHALL offer it where discovery advertised no scheme at all, a server advertising nothing being as likely to want nothing as to want something undiscovered.

### Requirement: A data command answers a named output type
A command returning data SHALL hand the printer a dedicated type named `<Domain><Target><Verb>Output`, deriving `Display`, `Serialize` and `JsonSchema`, with every public field documented. `pimalaya_cli::printer::Message` SHALL carry confirmations only: it serializes as one prose string, so a data command using it yields a `--json` payload no consumer can read.

A command whose two shapes are one invocation SHALL answer one type covering both, untagged so each shape serializes exactly as it would alone.

### Requirement: The JSON output of every data command has a schema
Every output type SHALL be registered in [json_schema.rs](../../src/json_schema.rs) under its CLI invocation path, hyphen-joined and prefixed `cardamum-`, and the `json-schema` command (aliased `json-schemas`) SHALL print one or write one file per entry. A registered key naming no command is a test failure, so the registry cannot drift from the tree.

A wire object the protocol crate exposes without a `JsonSchema` of its own SHALL be described as raw JSON rather than left out: the payload is the provider's to define, and claiming a shape we do not own would be worse than claiming none.

### Requirement: comfy-table is reached through the toolkit
Table rendering SHALL go through `pimalaya_cli::table` rather than a direct `comfy-table` dependency, so the toolkit owns the version every Pimalaya CLI draws with.

## MODIFIED Requirements

### Requirement: Path fields are shell-expanded
A configuration field naming a filesystem path SHALL expand `~` and environment variables as the value is deserialized, never at a call site, so a path written home-relative resolves against the real home rather than the current directory and no reader of the field can forget. An optional path SHALL carry `serde(default)` alongside its deserializer, an absent key never reaching it.

## REMOVED Requirements
