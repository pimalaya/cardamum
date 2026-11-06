---
cairn: change
id: pimdir-follows-storage-draft-09
status: landed
created: 2026-11-06
---

# The pimdir backend follows storage-draft-09

## Why

io-pimdir 5c18a02 (pushed, master, crate 0.9.0) follows pimdir 8c16bb1: storage-draft-09, sync-draft-08, search-draft-09 (io-pimdir cairn/log/2026-11-05-the-crate-follows-storage-draft-09.md). neverest, himalaya and calendula move to it, and cardamum reads and queues into the store neverest owns, so it reads the same draft.

## What

- `io-pimdir = "0.9"`, the `[patch.crates-io]` rev moved from 0c70144 to 5c18a02.
- io-pimdir 0.9 takes ical-rs 0.7, vcard-rs 0.6 and mime-rs 0.1 from crates.io: the ical-rs git patch (99b9273, there for io-pimdir's search alone) is dropped, and cardamum's own vcard-rs moves from 0.5 to 0.6, whose breaking changes (`VcardBinary::decode_base64`, the boxed `VcardValue::Adr`) touch no cardamum code.
- A store of storage-draft-08 is refused as `store-version`, never migrated; cardamum's reinstall message reads `FORMAT_VERSION` and needs no change. An index of search-draft-08 is refused as `index-version`.

## What the compiler does not see

- `PimdirSearchClock` and `PimdirSearchOptions` are no longer `Copy`: cardamum builds them per call and never copies them.
- `PimdirLoadSelection::Owing` and the flag bits are the owner's: cardamum reads through `PimdirReader` and stages no flag.
- `PimdirSearchClock::zone` stays unset: a card has no date (SEARCH §8.1), so `index-zone-unknown` is raised by no cardamum path.
- vcard-rs 0.6 folds a 3.0 or 4.0 line longer than 75 octets that it builds or edits: `card build`, `card create` and `card update` now write folded lines.
