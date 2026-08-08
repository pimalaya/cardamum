---
cairn: change
id: lib-version-bump
status: landed
created: 2026-08-09
---

# Update to the latest Pimalaya libraries

## Why

Cardamum was pinned to a dependency set several releases behind the rest of the org: io-pim-discovery 0.3, pimalaya-cli 0.1, vcard-rs 0.1, comfy-table 7. Two of those hold back features the wizard alignment needs (`compose_all_within` for a deadline-bounded discovery, `wizard::keyring` for the shared credential picker), and the io-webdav pin predated the naming-canon rename plus a batch of card-addressing fixes the CardDAV test reports had already flagged.

## What

Bump every Pimalaya dependency to its current version, with the three breaking migrations that implies.

**comfy-table v7 to v8**, forced in lockstep: io-pim-discovery 0.5 exists only to bump pimalaya-cli to 0.2, which exists only to bump comfy-table to v8, so a single version has to resolve across all three and cardamum's own direct dependency moves with them. v8 replaces the positional preset string with a typed `TableStyle` builder. Rather than break the user-facing `table.preset` option, the v7 spelling is kept and mapped onto the builder by a new shared/table.rs, ported from Himalaya, which already solved this.

**io-webdav to HEAD**, adopting the `Webdav`/`Carddav`/`Caldav` naming canon (every import changes, module paths untouched) and the id-handling fixes.

**vcard-rs 0.1 to 0.2**: the flattened re-exports moved onto their real module paths, so `tree::prop::VcardPropLens` becomes `tree::prop::lens::VcardPropLens`. None of the renamed items are used here.

The pimalaya-config git patch is dropped, 0.1.1 being released. io-webdav, io-pimdir and io-replica stay on git patches until their next release.

## Scope / non-goals

- No behavioural change beyond the truncation indicator (`...` becomes `…`, a comfy-table v8 default).
- The `table.preset` config contract is deliberately preserved rather than narrowed to an enum, matching Himalaya.
- A build with no backend feature at all still fails to compile (`BackendClient` has no variants). Pre-existing, out of scope, and arguably meaningless to support.
