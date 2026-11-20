---
cairn: change
id: the-contract-audit-rulings
status: landed
created: 2026-11-20
---

# The contract audit rulings

## Why

The CLI contract audit of 2026-11-20 (cardamum/calendula report, findings CRD-X1 to X7 and TWN-X1, X2) found shapes the 1.0 major freezes: capability names in the store's vocabulary rather than the unified commands', a `pimdir queue show` that is not the `queue list` row, a `pimdir queue list` listing creations alone, a `pimdir search` text cursor disagreeing with its JSON one, and uncoded refusals. The owner ruled on the audit's proposals the same day (A1 to A6, "Go for all"): one queue listing shape for the family (A4), TLS tables spelled as Himalaya's with an account-level table inherited field by field (A1), and pimalaya-cli's `account check` printing a list of accounts with `tls` on each backend row.

## What

- `account check` prints pimalaya-cli's list of accounts, its backend rows carrying `tls`; a bare `cardamum --json` is refused through pimalaya-cli.
- The TLS tables gain `client.cert` and `client.key`, and the account a `tls` table every network backend inherits field by field.
- `account capability` names the unified commands: `card.delete`, `addressbook.update`, `card.search.sort`...
- `pimdir queue list` prints `{actions, parked, errors}`, every pending row of every kind, an undecodable queue an item error; `pimdir queue show` prints the list row plus `state`, `appliedAt`, `refusals`, an unknown row failing `row-not-found`; `pimdir status` parked rows name `collection`.
- `pimdir search` text prints `Next page: --page-token <token>`; `card list` and `jmap contact-card query` code a bad token; the composer refused in a session and a source that is no vCard are coded.
- `addressbook create` on pimdir fails `addressbook-duplicate` before `capability-missing`; every protocol command documents exit 3; `accounts` is a hidden alias of `account`.
