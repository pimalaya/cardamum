---
cairn: change
id: the-v1-pass
status: active
created: 2026-11-18
---

# The v1.0.0 pass

## Why

The family analysis of 2026-11-15 (re-checked 2026-11-16) left cardamum lines that hold its next major: losses and wrong results (CAR-1, CAR-2, CAR-3, CAR-8, CAR-14), a security line (CAR-4), and contract shapes that break whoever fixes them after a major (CAR-11, TWN-3, TWN-4). The owner ruled on 2026-11-18:

- cardamum's next major is v1.0.0, as neverest and carillon: MIGRATION.md and the by-name `removed_key!` refusals go, the CHANGELOG marks breaking lines.
- CAR-11: `carddav delete <ADDRESSBOOK> <CARD-ID>` always deletes one card, the card id required; the collection delete moves to `carddav rmcol <ADDRESSBOOK>`, mirroring `mkcol`. `carddav put --if-none-match <value>` sends its value.
- TWN-3: a pimdir write into an address book no source of the store syncs is refused before queueing, coded `no-performer` (himalaya's code for the same case), as calendula does.
- TWN-4: a bare DAV `server` probes `.well-known` on both twins; a value that does not parse as a URL is refused, never prefixed with `https://`. With CAR-4, the `.well-known` redirect is followed over https only.

## What

One line per finding, each its own commit:

- **CAR-1.** `vdir item move` and `copy` onto a target holding an item of the same id fail coded `item-duplicate`, both items left as they were.
- **CAR-2.** A name flag rewrites the components of `N` it names and keeps its parameters; a 2.1 quoted-printable `N` is decoded before it is rewritten.
- **CAR-3.** vCard 2.1 quoted-printable values are decoded in the projection.
- **CAR-8.** The unified card verbs on vdir act on `.vcf` items alone: an `.ics` id is `card-not-found`, an update never creates a `.vcf` beside it.
- **CAR-14.** The composer writes its draft in a private directory, mode 0600, and removes it whatever the outcome.
- **CAR-4 with TWN-4.** The redirect of the `.well-known` probe is followed over https only; a `server` that does not parse is refused.
- **CAR-11.** `carddav delete` requires the card id; `carddav rmcol` deletes an address book; `--if-none-match` sends its value.
- **TWN-3.** The pimdir refusal is coded `no-performer`.
- **CAR-5.** On JMAP, Graph and People, `card read`, `update` and `delete` act only on a card the named address book holds; a failing test first.
- **io-pimdir 0.16.** The bump, search paging following its cursor.
- **v1.** `removed_key!` and MIGRATION.md go; a removed key is an unknown key (serde).
- **carddav-query-filter.** Its tasks are ticked but its delta was never folded: check the code against it, fold it, archive it.

## Out of scope

- TRI-3 (a missing blob answers `body-pending`), landed by another session the same day.
- card-search, which waits on io-pimdir's in-memory card matcher.
- The version bump in Cargo.toml, made at release time.
