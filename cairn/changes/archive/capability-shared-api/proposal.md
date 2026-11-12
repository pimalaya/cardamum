---
cairn: change
id: capability-shared-api
status: landed
created: 2026-11-12
---

# The unified API serves the union of the backends

## Why

Eighth change of the-family-standards (owner, 2026-11-12; calendula adopts it too). The unified API was the least common denominator, each backend refusing what it could not do with an ad hoc error, uncoded, once connected. Himalaya's capability union (himalaya capability-shared-api) declares what each backend serves, refuses an unserved verb before connecting with a code, and documents it in the help.

## What

- src/capability.rs: `Capability` (`addressbook.list|create|rename|delete`, `addressbook.description`, `addressbook.color`, `card.list|read|add|update|remove`, `card.update.if-match`, `card.remove.if-match`, pimdir STORAGE Annex B's names without the domain word, the collection read as the addressbook), `Support` at `full`, `partial`, `server` or `none` with its detail, one table per backend keyed by name and compiled into every build, seeded from the adapters' refusals: no colour on JMAP, Graph and People, no description on Graph and People, the default Contacts folder and the Contacts system group kept, no `--if-match` on JMAP, Graph, vdir and People deletes, no addressbook write on pimdir and its card writes queued, addressbook creation and deletion left to a CardDAV server that may forbid them.
- Each unified command calls `AddressbookClient::require` with the capabilities it reaches, flag-dependent ones included, before reading or connecting: an unserved one fails with the new `capability-missing` code, naming the backend, the capability and why. The adapters keep their own refusals as a second guard.
- `account capability` lists the capabilities of the backend the account resolves to (`--backend` honoured) with their level and detail, connecting nowhere: `{account, backend, capabilities: [{name, level, detail}]}`, `Columns` in text.
- Each unified leaf's `--help` ends with a generated `Backends:` footer: per capability, the backends by level, then each detail.
- A served verb prints exactly what it printed: the tables only refuse what an adapter refused already, now before connecting and coded.

## Scope / non-goals

carddav-query-filter (a unified card search) stays independent. pimdir's rows are static: reading what the store's sources declare, as Himalaya does, is left for when a contacts source declares less than the sync engines do today.
