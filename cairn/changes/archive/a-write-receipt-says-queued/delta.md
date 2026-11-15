---
cairn: delta
change: a-write-receipt-says-queued
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A queued write says it is queued
(commands) A unified write SHALL say whether it reached the server or was queued for a sync. On pimdir, `card create`, `card update` and `card delete` SHALL say the write is queued and that the next sync carries it, never that the card was created, updated or deleted. Under `--json`, every unified write SHALL print `queued`, the family's receipt key (pimalaya-cli output.md, "A write answers a receipt"), on every backend: `true` when a sync engine applies the write later (pimdir), `false` when it reached the server. `card create` prints `{id, queued}`, `card update` `{id, keptProperties, queued}`, `card delete`, `addressbook update` and `addressbook delete` `{id, queued}`, `addressbook create` `{id, name, queued}`, `name` the display name it was given, as Himalaya's `mailbox create` prints it.

#### Scenario: A card queued on pimdir
- GIVEN a pimdir address book a source syncs
- WHEN `cardamum --json card create -k <book> <vcard>` runs
- THEN it prints `{"id":"<link id>","queued":true}`, and the text form says the next sync creates the card

A queued creation SHALL get no placeholder id (`q<id>`, `0`, `""`): `card create` reports the card's link id, which `card read` does not accept until a sync applies the create and the card lists under its public id.

### Requirement: A data command answers a named output type
(commands) A command returning data SHALL hand the printer a dedicated type named `<Domain><Target><Verb>Output`, deriving `Display`, `Serialize` and `JsonSchema`, with every public field documented. A write with no payload of its own SHALL print a receipt rather than a sentence: a unified write `{id, queued}` (`WriteReceipt`), the id of the item acted on and whether a sync engine applies the write later, `addressbook create` `{id, name, queued}`, `card create` `{id, queued}`, `card update` `{id, keptProperties, queued}`, a write in a protocol namespace `{id}` (`IdReceipt`) or `{ids}` (`IdsReceipt`), the ids as they are after the write (a renamed collection's new one), and a `card build -o` `{path, size}` (pimalaya-cli's `output::SavedOutput`). `keptProperties` stays on `card update`: it names what a People update dropped and the server kept ("A partial write says so"), a result calendula's `event update` and `item update` print no counterpart of, so the twins align on `{id, queued}` and cardamum adds this one key. The sentence is the text form only; data a confirmation used to word (an ETag, member counts) is a key of its own.

Every runnable command SHALL be described in the `json-schema` registry, a command printing one of two types through both (`OneOf`), which a test checks over the clap command tree.

A command whose two shapes are one invocation SHALL answer one type covering both, untagged so each shape serializes exactly as it would alone.

### Requirement: Unified outputs carry no backend details
(backends) The output of a unified command SHALL NOT carry a field that only one backend fills. A backend detail worth showing SHALL be logged by that backend's adapter or shown by its own namespace. The receipt key `queued` is no such field: the family's receipts carry it on every backend (pimalaya-cli output.md), `false` wherever the write reached the server.

## REMOVED Requirements
