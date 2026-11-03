---
cairn: change
id: the-outputs-become-data
status: landed
created: 2026-11-03
---

# The outputs become data

## Why

pimalaya-cli 0.3 (its change every-output-is-data, landed 2026-11-03) removes `printer::Message`, so every write cardamum confirmed with a `{"message": ...}` sentence no longer builds, and asks every runnable command to have a schema.

## What

A write with no payload of its own prints a receipt: a shared write `{id, queued}`, a protocol-namespace write `{id}` or `{ids}`, a `card build -o` `{path, size}`; data a sentence held (an ETag, member counts) becomes a key. The registry moves to `JsonSchemaRegistry::new` and `insert`, describes every runnable command, a command printing one of two shapes through both (`OneOf`), and a test checks it over the clap command tree.

## Compatibility

Breaking for every `--json` caller of those writes, which read a `message` key before.
