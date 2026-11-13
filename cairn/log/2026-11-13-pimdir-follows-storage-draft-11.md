---
cairn: log
change: pimdir-follows-storage-draft-11
date: 2026-11-13
---

# The pimdir backend follows storage-draft-11

cardamum builds against io-pimdir at cd33f0e (storage-draft-11, sync-draft-11 and search-draft-11; spec at pimdir d8800fb), the commit following the fourteen lines draft-10 parked for the standard's owner. Capabilities **backends** and **commands** moved.

## What landed

- **Cargo.toml, Cargo.lock**: `[patch.crates-io]` pins io-pimdir to the git revision cd33f0e51c93a6276d3b2089e409780bcfbda006, the dependency line staying `version = "0.11"`. The patch waits on the io-pimdir release carrying draft-11: the crate is still 0.11.0 at that commit, the version crates.io holds for draft-10, and the entry is dropped as it publishes (packaging, "Released dependencies, patched only while unreleased").
- **No compile break**: cardamum builds no `PimdirSearchReads` by a struct literal (its one hand-built coverage takes `PimdirQuery::reads`), matches on no `PimdirMutation`, and acknowledges no intent, so `mark`, the refused `Move` and `unconfirm_action` needed no code.
- **Coverage** (src/pimdir/index.rs, SEARCH §8.2): `PimdirCardCoverage::bodiless` is summed over the kinds whose bodies a term of the query reads, as `missing` and `unparseable` are, joined by the kinds `PimdirSearchReads::counts_bodiless` names, where it summed every kind. `complete` stays `PimdirSearchCoverage::is_complete`, which no longer holds a bodiless card against a query of summary terms, so each count at rest still means `complete`: `pimdir count email:@example.org` over an address book listed and not fetched answers complete, `bodiless` 0, no line under it, where it answered incomplete naming the card. `counts_bodiless` is true for mail under `has:attachment` alone, a kind no cardamum query reaches behind `kind:contact`: it is called so the rule stays io-pimdir's, and changes no cardamum answer today.
- **The scope composes** (SEARCH §8.2): `scope` adds `kind:contact`, the account and `-k`'s address book beside the query's conjuncts, never around them, so a group that is a conjunct of the user's query (`(collection:team) offsite`, or the whole query in parentheses) now narrows the coverage as its terms do bare, io-pimdir reading through it. Two `collection:` or two `kind:` conjuncts intersect (io-pimdir retains the collections each names), so a group cannot widen the scope. No code.
- **An `@` in a text field** (SEARCH §8.3, §8.5): cardamum hands the query's words to `PimdirQuery::parse` and reads no `@` itself. `email:@example.org` stays an address, reading no body. No code.
- **A `move` over a conflict** (SYNC §7; STORAGE §15.2, §15.3): cardamum queues `add`, `update` and `remove` alone (src/pimdir/backend.rs), no command moving a card between address books, so no cardamum write meets the new park and nothing it tells the user moves. The spec says so beside the `update` and the `remove` a conflict parks.
- **Store version**: a store of storage-draft-10 is refused as `store-version`, an index of search-draft-10 as `index-version`, neither migrated. The reinstall message reads `FORMAT_VERSION`, so it names `storage-draft-11` unchanged.
- **Docs**: the spec folds below; README (the unread bodies of a search count when a term reads card bodies); MIGRATION.md (the draft a store is read under, the three counts at 0 under a query reading no body); CHANGELOG Unreleased, its pimdir lines being unreleased and so edited in place (the pin and the three drafts, storage-draft-11, the coverage line).

## Spec

- backends, modified: "A pimdir write over a conflicted card is parked" (cardamum queues no `move`), "A pimdir store of another version is reinstalled" (`storage-draft-11` at the pinned commit, its scenario a draft-10 store).
- commands, modified: "pimdir searches cards through the store's index" (search-draft-11, the scope beside the query, a scenario), "A pimdir card search says what it did not cover" (`bodiless` counted as `missing` and `unparseable` are, the rule io-pimdir's, a scenario).

## Tests

- `a_bodiless_card_counts_only_against_a_term_reading_it`, new: over the indexed fixture, whose Erin is listed and not fetched, `email:@example.org` and `kind:contact` are complete with `bodiless` 0 and no line, `has:phone` is incomplete with `bodiless` 1 and its line. Seen failing twice: on io-pimdir 0.11.0 from crates.io, before the pin (`complete: false, bodiless: 1` under `email:@example.org`), then on the pinned crate with the sum over every kind still in place (`bodiless` 1 for 0, the answer complete).
- `a_group_narrows_the_coverage_as_its_terms_do`, new: `collection:team offsite`, `(collection:team) offsite`, `(collection:team offsite)` and `((collection:team) (offsite))` list Dave alone with a complete coverage, and `-(collection:book) offsite` still counts Erin. Seen failing on io-pimdir 0.11.0 from crates.io, before the pin (`bodiless` 1 for 0 under `(collection:team) offsite`); the rule is io-pimdir's, so there is no cardamum code to put back.
- Changed, the rule having moved: the version fixtures name `storage-draft-10` and `search-draft-10`, the store and the index a draft-10 neverest wrote (`a_store_of_another_version_asks_for_a_reinstall`, `an_index_of_another_draft_is_refused_as_index_version`, `a_pimdir_failure_prints_the_code_io_pimdir_names`). They passed unchanged on the pin, draft-09 being refused too. No cardamum test stamps a later draft.
- Not guarded: the `counts_bodiless` half of the sum, no store a cardamum query reads holding a kind it is true for.
- 101 tests pass with every feature (99 before), 75 with `--no-default-features --features rustls-ring,pimdir` (73 before), each as `cargo test --bin cardamum`; `cargo fmt --check` is clean; `cargo check --all-features --tests` passes, and `cargo check --all-targets` for carddav, jmap, msgraph, gpeople, vdir and pimdir each alone with `rustls-ring`; clippy with `-D warnings` is clean with every feature and with pimdir alone. The suites under tests/ reaching no provider (capability, exit, first_run, mcp, repl) pass, 21 tests.

## Not in this entry

- The suites under tests/ reaching a provider, compiled and not run, and an end-to-end run on a store a draft-11 neverest synced: the owner's, once the stores are reinstalled.
- Dropping the patch, and the `version` the dependency names then: io-pimdir's CHANGELOG marks the follow breaking, so its release is likely 0.12.
- A change folder: this entry is the whole record, the intent having been given whole.
- The audit lines the CLIs still owe (pimdir cairn/changes/audit-2026-11-12/tasks.md), C2, C4 and C5, as the draft-10 entry left them; refusing a write over a conflicted card at staging, and a command resolving a conflict.
- The commit.
