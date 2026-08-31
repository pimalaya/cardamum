---
cairn: change
id: camelcase-json-keys
status: landed
created: 2026-08-29
---

# Delta

## ADDED Requirements

### Requirement: JSON output keys are camelCase
Every output type SHALL carry `serde(rename_all = "camelCase")`, so a `--json` key spelling more than one word reads `addressbookId` rather than `addressbook_id`. That is the convention of the wire formats the backends speak, and unlike a hyphenated key it needs no quoting in a `jq` path. The configuration vocabulary is a separate one and stays kebab-case.

A field renamed onto a provider's own wire key SHALL keep that spelling, a container attribute never reaching a field carrying its own `rename`: `@odata.nextLink`, `nextPageToken`, `contactGroups` and the JMAP `list` are the provider quoted back to the caller, not ours to restyle.

## MODIFIED Requirements

## REMOVED Requirements
