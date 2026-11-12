---
cairn: change
id: pimdir-honours-if-match
status: landed
created: 2026-10-04
---

# pimdir honours `--if-match`, and a delete can be gated

## Why

A caller that previews a change and writes it later (MOA, asking the user to confirm a contact edit or delete) has to know the card it writes over is still the one it showed. On pimdir `--if-match` was ignored, so MOA compared versions itself, and `card delete` had no guard at all. The sync engine's three-way merge protects the store, not the caller's preview.

## What

- The pimdir version of a card, already reported as `etag`, is the store's hash of its body with the pending queue folded in.
- pimdir `update_card` and `delete_card` refuse a write whose `if_match` is not that version, before anything is queued, with an error starting `Precondition failed`.
- `card delete` takes `--if-match`: CardDAV sends it, pimdir checks it, the other backends refuse it.
