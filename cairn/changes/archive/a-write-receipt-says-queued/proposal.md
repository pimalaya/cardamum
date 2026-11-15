---
cairn: change
id: a-write-receipt-says-queued
status: landed
created: 2026-11-15
---

# A write receipt says queued

## Why

`card create` printed `{id}` and `card update` `{id, keptProperties}`, their `queued` kept out of the JSON (2026-10-30, queued-stays-out-of-json), while `card delete` and the address book writes printed `{id, queued}`, and `addressbook create` printed `{id}` where Himalaya's `mailbox create` prints `{id, name, queued}`. pimalaya-cli's output.md makes `queued` the family's receipt key. Findings F7 and F8, task C3 of cli-alignment-2026-11-15, twin of calendula's K3; P2 rules `keptProperties` stays.

## What

- `card create` prints `{id, queued}`, `card update` `{id, keptProperties, queued}`, `addressbook create` `{id, name, queued}`, `queued` `true` on pimdir and `false` elsewhere.
- `keptProperties` stays on `card update`, the spec saying why.
- The `id` of a queued `card create` stays the card's link id (commands, "A queued write says it is queued"): pimalaya-cli's output.md would print `null` there, which this change does not take up.
- Breaking for a strict JSON consumer: MIGRATION.md says so.
