---
cairn: change
id: the-codes-follow-the-family
status: landed
created: 2026-11-15
---

# The codes follow the family

## Why

Four places where cardamum's failures read otherwise than its siblings' (cli-alignment-2026-11-15):

- A CardDAV 412 on `--if-match` and a `no-uid-conflict` were uncoded, where calendula codes `precondition-failed` and `item-duplicate` (finding F32, task C8).
- A card command with no `-k` over an account setting no `addressbook.default` failed uncoded (F34, C11).
- A vdir card not found read as io-vdir's own text (`Vdir item locate failed: item nope not found`, no `sources`) and a JMAP one as `JMAP ContactCard … not found`, where calendula prints its own sentence with the backend's text in `sources` (F18, C14).
- cardamum relayed 17 of the pimdir codes Himalaya relays (29), `store-not-current` and `sync-version` among the missing (F33, C15).

## What

- CardDAV: `is_duplicate_uid` is `item-duplicate`, a 412 `precondition-failed` (src/carddav/client.rs), for the unified and the protocol commands alike.
- A missing address book id is `addressbook-default-missing` (exit 1), the message naming `-k <ADDRESSBOOK-ID>` and `addressbook.default`, as calendula codes `calendar-default-missing` (its K13) and the family `account-default-missing`; the noun rule of pimalaya-cli's error.md gives the name. `usage-invalid` (exit 2) was the other reading, a command line missing an argument; the twins took the coded one.
- `Card `<id>` not found` on every backend, the backend's answer in `sources` (vdir and JMAP join CardDAV, Graph and People; pimdir keeps its sentence, having no text to add). Item errors already name no id in `error`.
- Himalaya's pimdir code table, all 29, in the same order; the 12 new codes join the vocabulary (additive).
