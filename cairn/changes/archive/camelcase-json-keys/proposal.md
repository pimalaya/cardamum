---
cairn: change
id: camelcase-json-keys
status: landed
created: 2026-08-29
---

# The JSON output keys are camelCase

The family is moving its `--json` output onto one key convention, and camelCase is the one the wire formats these tools wrap already speak: JMAP, Microsoft Graph and Google People all name their properties that way. Cardamum ended up in the middle of both worlds. Its output types carried no `rename_all` at all, so serde emitted the Rust field names verbatim and a listing answered `addressbook_id` next to a passthrough `nextPageToken` in the same document. Two conventions in one payload is one too many, and the snake_case half is the one nothing else in the stack uses.

A hyphenated key would have been worse still: `."sync-token"` has to be quoted in every `jq` path, where `.syncToken` does not.

Cardamum is 0.2.0, so the switch is made in one go, with no compatibility alias.

## What changes

Every type whose value reaches `printer.out` gains `#[serde(rename_all = "camelCase")]`, nested row and payload types included. Nine keys change spelling: `addressbookId`, `fnValue`, `keptProperties`, `addressbookHomeSet`, `displayName`, `syncToken`, `dataBytes`, `newState`, `hasMoreChanges`. Every command doc comment enumerating a key follows, the doc comment being the help page.

## What does not change

A field explicitly renamed to a provider's own wire key: `@odata.nextLink`, `@odata.deltaLink`, `nextPageToken`, `nextSyncToken`, `contactGroups`, and the JMAP `list`. A container `rename_all` does not reach a field carrying its own `rename`, which the regenerated schemas confirm. Those keys are the provider's vocabulary, quoted back to the caller, not ours to restyle.

The TOML configuration, which is a separate vocabulary and stays kebab-case.
