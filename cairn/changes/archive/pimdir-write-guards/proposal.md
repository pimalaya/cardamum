---
cairn: change
id: pimdir-write-guards
status: landed
created: 2026-08-09
---

# pimdir: a rename that renames the wrong thing, and three writes that never look first

The 2026-08-09 pimdir run ([pimdir-local.md](../../spec/testing/pimdir-local.md)) found four defects, all on the shared arm.

**PD1: `addressbook update -n <name>` rewrites the collection's id.** io-pimdir's `rename_collection(collection, new_id)` renames the identifier, as its parameter says; the backend calls it with the display name. So `-n "Contacts"` leaves `{"id": "Contacts", "name": "contacts"}`, every card stranded under an id nobody typed, and the account's `addressbook.default` matching nothing, so a plain `card list` reports zero contacts.

There is no local fix that keeps the flag working: io-pimdir exposes no display-name setter, the name being a column the sync writes from the server. A local rename would be exactly what this backend refuses to do elsewhere, a change no sync will ever carry. The honest move is to refuse it, as `delete_addressbook` already refuses for the same reason.

**PD2, PD3, PD4: three commands that never check what they address.** `card list -k <unknown>` prints an empty table and exits 0, so an unknown addressbook is indistinguishable from an empty one. `card create -k <unknown>` creates the collection, so a typo invents an addressbook and hides the card in it. And an item whose blob is missing lists as a blank row, because the listing reads the body with `unwrap_or_default()`, while `get_card` on the same item correctly reports "Body blob missing".

## What changes

- `update_addressbook` refuses a rename with a message naming the reason, joining the description and colour it already refuses. Every field of `addressbook update` is now refused on pimdir, and each says why.
- A private guard resolves a collection before the shared reads and writes touch it, so `card list` and `card create` on an unknown addressbook fail with "Addressbook `x` not found" instead of inventing or pretending.
- The listing falls back to the summary preview when a body is missing, as the unhydrated path already does, and logs a warning: a store whose blob is gone is inconsistent, and a blank row says nothing.

## What does not change

The writes that do work: a card written into a synced collection still stages as a dirty placement, a tombstone or a baseless create, which the run verified against a seeded store.
