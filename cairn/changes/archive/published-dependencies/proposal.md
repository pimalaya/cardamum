---
cairn: change
id: published-dependencies
status: landed
created: 2026-11-12
---

# Build on the published dependencies

## Why

Every revision cardamum pins through `[patch.crates-io]` is now inside a release (checked with `git merge-base --is-ancestor` on 2026-11-12): io-http e181530 in 0.6.0, io-msgraph 2ae2637 in 0.7.1, io-pimdir 5c18a02 in 0.10.0, pimalaya-cli f976e40 in 0.3.2. The pins also split the tree: Cargo.lock holds io-http 0.5.0 (the pin) beside 0.6.0 (under io-webdav 0.7), and vcard-rs 0.5.2 (under io-jmap 0.6, io-gpeople 0.5 and io-msgraph at the pin) beside 0.6.0. It comes first in the-family-standards so any API move surfaces before the large edits, as calendula's published-dependencies did.

## What

- Cargo.toml requires io-msgraph 0.7.1, io-pimdir 0.10 and pimalaya-cli 0.3.2 (build and normal) and drops `[patch.crates-io]`; io-jmap moves from 0.6 to 0.7 and io-gpeople from 0.5 to 0.6, the releases on io-http 0.6 and vcard-rs 0.6, so the tree holds one of each.
- What the releases change for cardamum: io-jmap 0.7 adds `AnchorNotFound`, `UnsupportedSort` and `UnsupportedFilter` to `JmapMethodError` (read as `Unknown` before); io-http 0.6 fails a response head over 64 KiB, a body over 256 MiB or a truncated body with a typed error on every backend, where the JMAP and People paths could not; io-gpeople 0.6 and io-msgraph 0.7 fold vCard lines over 75 octets through vcard-rs 0.6. Any compile break from these moves is fixed here, no behaviour of cardamum's own moving.
- It completes A2 of cli-alignment-leftovers and the pin tasks of pimdir-release-api-followups, both moved here.
- CHANGELOG: the clauses naming git revisions and "to be dropped for releases" become the released versions.

## Scope / non-goals

- No `default-features = false` pass, no base64 swap, no pimalaya-cli 0.4: those are dependencies-trimmed and the-shared-kit.
- io-jmap 0.7's own `vcard` feature is not adopted: the JMAP projection stays on vcard-rs's `jscontact` codec (src/jmap/project.rs).
