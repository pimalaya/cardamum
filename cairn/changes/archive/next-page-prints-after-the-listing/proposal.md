---
cairn: change
id: next-page-prints-after-the-listing
status: landed
created: 2026-11-15
---

# Next page prints after the listing

## Why

pimalaya-cli 0.6.0 (owner ruling D27) prints a `Listing`'s next page after its items, `Next page: --page <n>` one blank line below them and above the item errors, and aligns the header values two spaces past the longest label. Task X14 of cli-alignment-2026-11-15: cardamum moves to pimalaya-cli 0.6.0, io-gpeople 0.8.1 and io-msgraph 0.9.2.

## What

- Cargo.toml requirements `0.6.0`, `0.8.1`, `0.9.2`; Cargo.lock moved for those crates alone.
- `card list`, `pimdir search`, `addressbook list`, `pimdir queue list` and `account capability` print the new frame, no cardamum code changing; the tests asserting the old text follow.
