---
cairn: change
id: pimdir-local-backend
status: landed
created: 2026-08-09
---

# pimdir local backend (read plus staged writes)

## Why

Cardamum could read and write remote address books and a local vdir, but not a **pimdir** store: the SQLite-indexed, content-addressed local cache the sync engine (Neverest, over io-replica and io-pimdir) populates. Reading the same store the sync writes gives an indexed, offline, provider-agnostic address book with no second copy and no format bridge, and it is what the mobile apps already do.

Himalaya landed the mail-side equivalent (himalaya/cairn/changes/pimdir-cache-backend); this is its contacts twin.

## What

A new `pimdir` feature and `src/pimdir/` (client.rs, card.rs, backend.rs, hash.rs), a `BackendClient::Pimdir` variant placed local-before-network, a `PimdirConfig` block, an `account check` arm, and marker-based detection in the wizard's local-path branch.

Reads observe the shared items through io-pimdir's client read API: `list_addressbooks` (collections of kind `text/vcard`) and `list_cards` (keyset-paged in the store's contacts order), with `get_card` reading the content-addressed blob.

Writes stage io-replica mutations a later sync pushes, never raw SQL: `create_card` to `Add`, `update_card` to `Edit` (the mutable-content axis io-replica gained for contacts, which is why contacts work here at all), `delete_card` to `Remove`. A write is attributed to the configured `pimdir.source`; on a store never synced as that source it fails loudly rather than staging a change no sync carries.

## Scope / non-goals

- **Cache semantics, not a live backend**: no fetch-on-demand, since a pure reader has no remote and hydration is the sync's job. An unhydrated card lists from its stored summary as a preview vCard, and `get_card` refuses it outright so the preview can never pass for the document of record.
- **Shared subcommands only.** pimdir is a store, not a protocol; its operator surface is the separate `pimdir` binary in io-pimdir.
- **`delete_addressbook` is unsupported**, and most of `update_addressbook` with it: io-pimdir exposes no collection removal, io-replica has no collection-level mutation, and pimdir stores no description or colour. Both fail with a message naming the limit rather than silently no-op.
- **Source alignment is a deployment concern** (single writer plus source identity), documented and guarded, not solved here.
- The derivations duplicate Neverest's `text/vcard` kind rather than sharing it. They must agree byte for byte, and there is no crate to share them through; the duplication is deliberate and flagged in both files.
