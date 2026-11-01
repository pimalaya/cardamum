---
cairn: change
id: pimdir-status-shows-parked-actions
status: landed
created: 2026-10-30
---

# pimdir status shows parked actions

## Why

Since pimdir-follows-storage-draft-05 (2026-10-30), a pimdir `card update` made from the card it read queues its `update` with `if_match`, and the store's owner parks the row when a sync replaced the body before the drain. The write said it was queued, then never lands, and cardamum has no view of it: the spec leaves the parked row to the sync engine (cairn/spec/backends.md, "A queued pimdir edit names the body it replaced"), so today only neverest shows it. A duplicate create or a push the server rejects parks the same way. The pimdir stable-release plan (2026-10-30, Apps, Phase A, Common) asks every app for parked actions in `pimdir status`; calendula and unfog have it, cardamum has no `pimdir` namespace at all (cairn/spec/commands.md, "Specific commands mirror their protocol").

## What

- A `pimdir` command family, gated on the `pimdir` feature, with one subcommand, `status`, mirroring calendula's (calendula/src/pimdir/status.rs): the account read, every account the store groups collections under, one row per address book of that account (how many cards it holds, how many carry a local body, how many creations are queued), and every action the store's owner parked in those address books: its queue row (ROW, the id the store's own operator tooling retries or cancels it by), the address book, the action kind and the error that parked it.
- The JSON output is `PimdirStatusOutput`, camelCase: `{"account", "accounts", "addressbooks": [{"id", "name", "total", "hydrated", "queued"}], "parked": [{"row", "addressbookId", "action", "error"}]}`, registered as `cardamum-pimdir-status`. calendula's `calendars` and `calendarId` become `addressbooks` and `addressbookId`, the vocabulary of each CLI's shared API.
- Scoped to the account read (`pimdir.account`, or the one the store groups), so another account's parked rows never show.
- The rest of the store's operator surface (retrying or cancelling a parked row, the retained items) stays with the `pimdir` binary io-pimdir ships.

## Not in scope

- Listing every pending action: like calendula's, the report counts the queued creations (which list nowhere else) and lists the parked rows; a queued update or removal already reads back through the shared commands.
