---
cairn: log
change: the-outputs-become-data
date: 2026-11-03
---

# The outputs become data

The user bumped pimalaya-cli and pimalaya-config to 0.3, io-jmap to 0.6, io-pim-discovery to 0.9 and the devshell. pimalaya-cli 0.3 removes `Message`, which 22 writes printed and the `card build -o` path too, and requires `Printer::event` and `out_bytes`.

src/shared/receipt.rs holds `WriteReceipt` (`{id, queued}`, its sentence worded by the write's kind as before, `Card <id> deletion queued; the next sync applies it`), `IdReceipt` (`{id}`) and `IdsReceipt` (`{ids}`, `jmap contact-card copy`). `carddav put` prints `{id, etag}` (`CarddavPutOutput`), `gpeople contact-group members` `{id, added, removed}` (`GpeopleMembersOutput`), `card build -o` `{path, size}`. The registry moves to `JsonSchemaRegistry::new`, registers each, `card build` and the two JMAP updates through `OneOf`, and `every_command_is_described` checks it over the clap tree: it listed exactly the 22 writes before they were registered. Clippy 1.99 asked `carddav report query` to read its filter triples through `as_chunks`, which also drops the unreachable arm.

Spec: commands (A data command answers a named output type).

`cargo test --all-features` 87 passed, clippy clean. Left: the rest of cardamum's line in pimalaya-cli's ledger, listed in the change's tasks.md.
