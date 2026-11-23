---
cairn: tasks
change: follows-storage-draft-17
---

# Tasks

Run top to bottom. Every cargo command: `nix develop --command`, `CARGO_TARGET_DIR` on /home, `-j 4`, `taskset -c 4,5,20,21`. Gate G = `cargo fmt --check`, `cargo clippy --locked --all-targets <set> -- -D warnings` over the 11 sets of .github/workflows/tests.yml, `cargo test --locked --all-features` (366 passed, 19 ignored, plus the tests this change adds), `cargo test --locked --no-default-features --features rustls-ring,vdir`, `cargo test --locked --no-default-features --features rustls-ring,carddav --test account`. One unsigned commit per gated step (frame §4).

## A. Dependency and build

- [x] A1 Cargo.lock: `cargo update -p url` (2.5.8 to 2.5.9, form_urlencoded 1.2.3); gate `cargo deny check` all four ok (advisories fail at HEAD, I4). Commit `build(deps): move url to 2.5.9`.
- [x] A2 Cargo.toml: the frame's `[patch.crates-io]` io-pimdir pin; features `carddav`, `jmap`, `msgraph`, `gpeople`, `vdir` take `io-pimdir/contact` in place of `io-pimdir/search`, `pimdir` adds `io-pimdir/contact`; `[dev-dependencies] io-pimdir = { version = "0.19.1", default-features = false, features = ["indexer"] }`. Let the first build resolve the lock: never `cargo update -p io-pimdir`, which flips five windows-sys entries (I1). Gate: lock diff is io-pimdir's source, libc added, nothing else moved; `cargo tree -e normal --no-default-features --features rustls-ring,carddav -i rusqlite` matches nothing (I3).
- [x] A3 Nix: `nix build .#default.cargoDeps` vendors the git rev (I4); one `nix build .#default --cores 4 --max-jobs 1`; package.nix unchanged (its SQLite rule still holds).

## B. Compile breaks (I2)

- [x] B1 `PimdirSearchSort::Name` to `Title`: src/pimdir/search.rs:204, src/shared/card/search.rs:381, tests src/pimdir/index.rs:560, :902. The `--sort` value shown becomes `title`, `name` a hidden alias (D2), with a test that `--sort name` still parses; CHANGELOG Changed.
- [x] B2 src/pimdir/backend.rs:370 `row_target`: `PimdirAction::SetProperty { seq, .. }` joins the `seq` arm.
- [x] B3 src/shared/card/search.rs:12, :172, :249, src/pimdir/index.rs:836, :875: closed by A2's `contact`, no code.
- [x] B4 tests: src/pimdir/index.rs:348, :424 closed by the dev `indexer`; `server_object_id: None` at src/pimdir/backend.rs:2051, :2177, src/pimdir/index.rs:437.
- [x] B5 Gate G at 366/0/19 (the probe's, I3). Commit `build!: follow io-pimdir 7f16b27 (storage-draft-17)`.

## C. Format strings and refusals

- [x] C1 The byte patches (src/pimdir/backend.rs:2524, :2551, src/pimdir/index.rs:1133) stay: draft-11 and draft-99 have draft-17's length and pass (I5). No code.
- [x] C2 A draft-15 store (H0's c10k-d15) read by the new binary: `store-version`, io-pimdir's message quoted in the log entry (I6).

## E. Inherited behaviour, pinned by tests

- [x] E1 tests/search.rs: `a_reserved_form_is_refused_on_every_backend` (vdir: `~alice` `query-refused` naming `reserved-form`, `"~alice"` accepted); src/pimdir/index.rs the same on `pimdir search` and `count`. Fails on 0.19.1 (answered, exit 0, I7).
- [x] E2 tests/search.rs `a_label_is_a_store_term`: vdir `label:x` and `property:a/b=c` coded `query-unsupported`; src/pimdir/index.rs `a_label_finds_its_card`: `PimdirStore::add_property(seq, "pimdir/label", "vip")`, refresh, `label:vip` lists the card. Fails on 0.19.1 (`unknown-field`, I7).
- [x] E3 src/pimdir/index.rs `the_new_date_forms_are_accepted`: `date:30min`, `date:2026-08-01T10:00`, `mid:x` parse and answer (0.19.1: refused, I7).
- [x] E4 src/pimdir/backend.rs `a_set_property_row_names_its_card`: a producer queues `SetProperty`; `pending_rows` gives kind `set-property`, `itemId`, `fullName`, `linkId` null; `queue show` and `cancel` take it.
- [x] E5 src/pimdir/index.rs `the_matcher_answers_the_hits_of_the_index` gains a card with no `FN` and one whose name folds (`Émile`), both orders compared.
- [x] E6 Help and docs: `card search` and `pimdir search` doc comments (src/shared/card/search.rs:33-55, src/pimdir/search.rs:37-60) name `label:`, `property:`, `folder:` among the store terms and say quoted `~`, `/`, `{`.
- [x] E7 Gate G. Commit `feat!: the search follows search-draft-17`.

## G. Bugs

- [x] G1 Parked on needs-io-pimdir.md N1: an undecodable queue row hides its address book's rows.
- [x] G2 A query refusal's text reads `Error: [unknown-field] Error: Query refused` (TRI-7, still at HEAD, I7): src/shared/query.rs:54 `refusal_report` drops the doubled word, as himalaya's pimdir search does. Test `a_refusal_names_error_once` fails before. Commit `fix(search): ...`.
- [x] G3 Re-checked closed at HEAD, nothing owed: CAR-1 to CAR-14 (cardamum logs 2026-11-18, 2026-11-19), TRI-1, TRI-3, TRI-8, audit C4, C5 (inventory I8).

## H. Performance

- [x] H0 Fixtures (I9): `git archive` io-pimdir v0.19.1 and 7f16b27 into the lane scratch; the harness ~/code/pimalaya/.pimdir-migration/cardamum/scripts/cardfix (main.rs, two Cargo.toml naming each tree's io-pimdir and spec/generator) runs `profile::count(seed, items)`, keeps the contact collections and items, `writer::materialise`, then `PimdirIndex::refresh`. c10k: seed 7, 100,000 items (10,055 cards, account `home` 6,965); c100k: seed 7, 1,000,000 items. Account `home`, book `home/contacts`. A vdir copy of c10k's `home` bodies for H8.
- [x] H1 Release binaries: HEAD (0.19.1) and the B5 tree; three copies each on ext4; A/A within 1%; hyperfine under the measure lock on CPU 15, old and new interleaved, each on its own draft's store.
- [x] H2 Measures, 10k and 100k: `card list -k home/contacts` page 1 and `--page 200`; `card read <id>`; `pimdir search alice` page 1 and `--page 5`; `pimdir count alice`; `card search alice`; `pimdir status`; `card update --if-match` on a store copy restored per run; H8 `card search alice` over the vdir.
- [x] H3 Acceptance: each new median at most old ×1.05 or old + 1 ms, whichever is larger; at 100k, every page-1 read (list, read, search, count) under 50 ms. A miss is a finding for the io-pimdir lane, with its command. Known misses at 10k (I10): `card list --page 200` and `pimdir status`, filed as N3 and N4; they do not block the step, the report names them.
- [x] H4 Candidate fixes, each kept only if H2 shows it beyond noise: one `PimdirIndex` held across `search_page`'s walk (src/pimdir/search.rs:136-190 opens it per page through `index`, src/pimdir/index.rs:90); `scan` reading `min(SCAN_BATCH, skip + take)` rows per batch (src/pimdir/backend.rs:799; page 1 reads 500 rows for 25).
- [x] H5 Final tables re-run in a quiet window; inventory and the log entry carry them.

## I. Docs

- [x] I1 Fold delta.md into cairn/spec (backends, commands, packaging); backends.md:216 and commands.md:459 name draft-17.
- [x] I2 CHANGELOG Unreleased: BREAKING store refused and start over, the reserved forms, `label:` and `property:`, the date forms; Changed: features (no ical-rs, mime-rs in the binary).
- [x] I3 README: the start-over note (old neverest syncs until nothing is queued, move the store aside, new neverest syncs anew; the index is neverest's). Man pages and completions regenerate from the doc comments.
- [x] I4 cairn/log/2026-11-2x-follows-storage-draft-17.md; `sh cairn/verify.sh`.

## J. Tests beyond the suite

- [x] J1 Cross-app, after neverest's lane lands 7f16b27: neverest syncs a local Radicale address book (under the `servers` lock) into a scratch store; cardamum `card list`, `pimdir search`, `card add`; neverest syncs again; Radicale holds the card, `pimdir queue list` is empty.
- [ ] J2 Live (owner grant): `cargo test --test google -- --ignored anonymous::`; the credentialed suites as the owner runs them weekly.

## K. Release readiness (the owner's)

- [ ] K1 io-pimdir 0.20.0 published by its tag workflow; pimdir tags if the owner cuts them.
- [ ] K2 Every CLI (neverest, himalaya, cardamum, calendula, carillon) on that one revision before any release.
- [ ] K3 Swap the pin for `io-pimdir = { version = "0.20.0", ... }`, drop `[patch.crates-io]`, gate G; then cardamum's own release (v1.0.0) by its tag.

## L. Report

- [ ] L1 report.md: what landed, the parked items, the measures, decisions D1 to D7 for the owner.
