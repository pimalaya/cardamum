---
cairn: log
change: pimdir-follows-storage-draft-09
date: 2026-11-06
---

# The pimdir backend follows storage-draft-09

cardamum builds against io-pimdir 5c18a02 (crate 0.9.0; storage-draft-09, sync-draft-08, search-draft-09), the rev neverest, himalaya and calendula pin. Capabilities **backends** and **commands** moved.

## What landed

- **Cargo.toml**: `io-pimdir = "0.9"`, the `[patch.crates-io]` rev 0c70144 to 5c18a02, the ical-rs git patch (99b9273) dropped since io-pimdir 0.9 takes ical-rs 0.7 from crates.io, and vcard-rs 0.5.1 to 0.6. Nothing in src/ needed porting: cardamum never copies `PimdirSearchClock` or `PimdirSearchOptions`, never loads a selection, stages no flag, and uses neither `VcardBinary::decode_base64` nor `VcardValue::Adr`.
- **Cargo.lock**: io-pimdir 0.9.0 at 5c18a02; ical-rs 0.6.1 (git) to 0.7.0, mime-rs 0.0.1 (git) to 0.1.0, quoted-printable-codec 0.0.1 (git) to 0.1.0, all from crates.io; the git base64-codec 0.0.1 and simd-cpu 0.0.1 out, leaving their crates.io 0.1.0 alone; hashlink 0.12.2 in (io-pimdir's statement cache); vcard-rs 0.6.0 in for cardamum and io-pimdir.
- **Store version**: a store of storage-draft-08 is refused as `store-version`, never migrated. cardamum's reinstall message reads `FORMAT_VERSION`, so its wording is unchanged and names `storage-draft-09`. An index of search-draft-08 is refused as `index-version`.
- **Zone**: `PimdirSearchClock::zone` stays unset, so a query's dates read in UTC (SEARCH §8.1); a card has no date and every cardamum query carries `kind:contact`, so this changes no answer, and `index-zone-unknown` is raised by no cardamum path.
- **vcard-rs 0.6 folding**: a 3.0 or 4.0 line longer than 75 octets that `card build`, `card create` or `card update` builds or edits is now folded (checked by hand with `card build --note` of 120 octets). No spec states line shape, so none moved.
- **Spec**: backends "A pimdir store of another version is reinstalled" names `storage-draft-09` at io-pimdir 5c18a02 and its scenario `storage-draft-08`; commands "pimdir searches cards through the store's index" names search-draft-09 §8. CHANGELOG Unreleased says the same.

## Tests

- `a_store_of_another_version_asks_for_a_reinstall` rewrites `FORMAT_VERSION` into `storage-draft-08` (was `storage-draft-07`), the store a draft-08 neverest wrote; `an_index_of_another_draft_is_refused_as_index_version` rewrites io-pimdir's `sql::search::FORMAT_VERSION` into `search-draft-08` (was the literal `search-draft-08` into `search-draft-07`, which failed at the new pin: no draft-08 in the index); the error-code fixtures in src/error.rs name the draft-08 versions. Both tests fail against the 0c70144 pin (Cargo.toml and Cargo.lock restored to HEAD for the run), where a draft-08 store and index open.
- 104 tests pass with every feature and with the default features, 71 with `--no-default-features --features rustls-ring,pimdir`; `cargo check --all-targets` passes for carddav, jmap, msgraph, gpeople, vdir and pimdir each alone with `rustls-ring`, and for the default; clippy is clean with every feature and with pimdir alone.

## Not in this entry

- vcard-rs 0.5.2 stays in the tree beside 0.6.0: io-gpeople 0.5 and io-msgraph at 2ae2637 (the patched rev) require 0.5. io-gpeople 0.6.0 and io-msgraph 0.7.0 (60cb397) take vcard-rs 0.6; moving cardamum to them is its own change. Cards cross both crates as text, so the two versions never meet in a type.
- The end-to-end run on a scratch store a draft-09 neverest synced, and the commit.
