---
cairn: change
id: follows-storage-draft-17
status: active
created: 2026-11-23
---

# Inventory

Measured on 2026-11-23 at cardamum 22c0bd5 (io-pimdir 0.19.1) and io-pimdir 7f16b27, under ~/code/pimalaya/.pimdir-migration/cardamum/ (scratch: `src/` the snapshot with the pin alone, `base/` the snapshot untouched, `probe/` the pin plus tasks A2 and B, logs in `logs/`), CPUs 4,5,20,21, while other lanes built. Lines marked "read" are from code, unverified.

## I1. The lock

The pin resolved by the first build changes io-pimdir's source, adds libc and drops ical-rs and mime-rs (no edge left without the dev-dependency). `cargo update -p io-pimdir` instead also flips errno, rustix, rustls-platform-verifier, tempfile and winapi-util from windows-sys 0.61.2 to 0.52.0 (the trap the contract audit met with pimalaya-cli).

## I2. Compile breaks, pin alone (logs/pinned-all-features-build.log)

`cargo build --all-features --all-targets`: 6 errors in the binary, 15 in the test target (the 6 included).

| Site | Error | Cause |
| --- | --- | --- |
| src/shared/card/search.rs:12 | E0432 `io_pimdir::search::card` | the matcher is under `contact` |
| src/shared/card/search.rs:172 | E0599 `store_terms` | same |
| src/shared/card/search.rs:249 | E0599 `match_cards` | same |
| src/shared/card/search.rs:381 | E0599 `PimdirSearchSort::Name` | renamed `Title` |
| src/pimdir/search.rs:204 | E0599 `PimdirSearchSort::Name` | same |
| src/pimdir/backend.rs:370 | E0004 `SetProperty` not covered | new action |
| src/pimdir/index.rs:348 | E0432 `PimdirIndexRefreshOptions` (test) | under `indexer` |
| src/pimdir/index.rs:424 | E0599 `PimdirIndex::open` (test) | under `search` and `indexer` |
| src/pimdir/index.rs:437, backend.rs:2051, :2177 | E0063 `server_object_id` (tests) | new field |
| src/pimdir/index.rs:560, :902 | E0599 `Name` (tests) | renamed |
| src/pimdir/index.rs:836, :875 | E0432, E0599 (tests) | `contact` |

Clippy, the 11 CI sets, pin alone: 10 fail, `--no-default-features --features rustls-ring` (no backend) passes; a single network backend or vdir fails on the three `contact` errors (logs/clippy-pinned).

## I3. The probe (tasks A2 and B applied)

- `cargo test --all-features`: 366 passed, 0 failed, 19 ignored; the same test names as the baseline (logs/probe-test-all-features.log).
- Clippy `-D warnings`, the 11 sets: all clean (logs/clippy-probe). CI steps: vdir alone and `carddav --test account` pass; pimdir alone 136 unit tests pass; `cargo fmt --check` clean (logs/probe-ci-steps.log).
- `cargo tree -e features,normal -i io-pimdir`: carddav takes `contact` alone, pimdir `client`, `contact`, `search`; no rusqlite in a carddav or vdir build, dev edges included; ical-rs and mime-rs only through the dev-dependency.

## I4. Nix and cargo deny

- `nix build 'path:<probe>#default.cargoDeps'`: cargo-vendor-dir holds io-pimdir-0.19.1 from rev 7f16b27 with its source replacement (pimalaya/nix `importCargoLock` with `allowBuiltinFetchGit`). A full `nix build` was not run (lane CPU budget); neverest's Releases run 44950728447 (2026-11-21) built the same mkDefault with an io-pimdir git pin.
- `cargo deny check`: sources ok (deny.toml allows the pimalaya org), bans and licenses ok; advisories fail at HEAD and on the probe alike, RUSTSEC-2026-0592 (url 2.5.8); `cargo update -p url --dry-run` gives 2.5.9 and form_urlencoded 1.2.3 under the 1.89 MSRV.

## I5. Baseline and format stamps

- Baseline, base/ unchanged: `cargo test --locked --all-features` 366 passed, 0 failed, 19 ignored (285 unit), as the 2026-11-21 log.
- No SQL text in src or tests (0 hits); the byte-patch tests (`storage-draft-11`, `storage-draft-99`, `search-draft-11`, each draft-17's length) pass on the probe.

## I6. Stores across drafts

- The 0.19.1 binary on a draft-17 store: `store-version`, "Pimdir store is storage-draft-17, written by a later sync engine, and cardamum reads storage-draft-15 alone: upgrade cardamum".
- The probe on a draft-15 store (H0's c10k-d15): `store-version`, "Pimdir store is storage-draft-15, written by an earlier sync engine, and cardamum reads storage-draft-17 alone: a store is never migrated, so sync it with the engine that wrote it until nothing is queued, then move its directory aside and sync a new store". On the draft-17 twin, `pimdir count kind:contact` answers 6,965, as the 0.19.1 binary on the draft-15 one.

## I7. Query behaviour (vdir account, `card search --json`)

| Query | 0.19.1 | 7f16b27 |
| --- | --- | --- |
| `~alice` | answered, exit 0 | `query-refused`, `reserved-form` at 0..6 |
| `"~alice"` | answered | answered |
| `/home` | answered | `reserved-form` at 0..5 |
| `category:{x}` | answered | `reserved-form` at 9..12 |
| `label:x`, `property:a/b=c` | `unknown-field` | `query-unsupported` (vdir) |
| `mid:x` | `unknown-field` | answered, no hit |
| `date:30min`, `date:2026-08-01T10:00` | `invalid-date` | answered, no hit |
| `elodie` | `Élodie Martin` | the same |

On the draft-17 store `pimdir search -- ~alice` is refused alike.

Text refusal at HEAD: `Error: [unknown-field] Error: Query refused` (TRI-7).

## I8. Open lines triaged

| Line | Source | Verdict |
| --- | --- | --- |
| `pending_actions` fails a collection on one bad row | the-freeze-plan tasks.md:68, contract audit handoff item 8 | open, needs-io-pimdir N1, task G1 |
| TRI-7 doubled `Error:` | CLI analysis plan.md:229 | open, reproduced (I7), task G2 |
| TRI-9 offset paging | plan.md:231 | open: `pimdir search --page N` runs N searches, each opening the index (read: src/pimdir/search.rs:136-190, src/pimdir/index.rs:90); page 5 at 19 ms (I10); task H4 |
| TRI-1, TRI-3, TRI-8 | plan.md:223-230 | closed for cardamum (kind filter; missing body lists and reads `body-pending`, 2026-11-18; `store-account-*`, 2026-11-19) |
| CAR-1 to CAR-14 | plan.md:339-352 | closed (logs 2026-11-18, 2026-11-19); CAR-12's `pimdir.account` point by `a_configured_account_opens_a_store_holding_nothing` |
| audit C4, C5 | pimdir audit-2026-11-12 tasks.md:105-106 | C4 ticked; C5's cardamum copies dropped 2026-11-19, its io-pimdir tick waits on himalaya and calendula |
| cli-alignment open lines | ledger tasks.md:17, 178, 232, 255, 264, 267 | none on cardamum's pimdir surface |
| GitHub | `gh issue list` | #25 alone, card-search's |

## I9. Fixtures (H0)

The generator's count profile is unchanged between v0.19.1 and 7f16b27, so one seed gives one model at both drafts. Seed 7, 100,000 items, contacts kept: 10,055 cards in six books (home/contacts 5,511, home/colleagues 1,454), written in 11 s (draft-15) and 24 s (draft-17), indexed current; 60 and 63 MB. The 0.19.1 binary reads the draft-15 one (`pimdir count kind:contact` 6,965, `pimdir status`): an earlier-draft fixture exists, so the baseline needs no sync of its own.

## I10. Indicative timings

Release binaries, default features, copied to ext4 (bin/cardamum-base at 22c0bd5, 14,092,496 bytes; bin/cardamum-new the probe, 13,996,368 bytes), hyperfine 1.20 `-N`, 3 warm-ups, 20 to 30 runs, pinned to CPU 5 while other lanes built, each binary on its own draft's c10k store, account `home`. Indicative, not H's final tables. A/A (a second copy of the new binary) within 1% on `pimdir status`, 4% on page 200.

| Command | 0.19.1 | 7f16b27 |
| --- | --- | --- |
| `card list -k home/contacts` | 5.9 ms | 4.1 ms |
| `card list -k home/contacts --page 200` | 21.5 ms | 26.7 ms |
| `card read -k home/contacts 3153` | 3.9 ms | 1.9 ms |
| `pimdir search alice` | 29.2 ms | 5.5 ms |
| `pimdir search --page 5 alice` | 131.0 ms | 19.3 ms |
| `pimdir count alice` | 24.8 ms | 3.7 ms |
| `card search alice` | 30.0 ms | 5.7 ms |
| `pimdir status` | 26.0 ms | 34.7 ms |

The two regressions are the whole-collection walks through `list_summaries` (needs-io-pimdir N3, N4).

## I11. Final tables (H5)

Measured on 2026-11-23 at the end of the lane: old is the 0.19.1 binary (22c0bd5) on the draft-15 stores, rc the release build at io-pimdir 17d4ea8 (0.20.0 RC) on the draft-17 twins. Three copies of each binary on ext4, interleaved, hyperfine 1.20 `-N -w 3 -r 20` on CPU 15 under the measure lock, account `home`, book `home/contacts`, card 3153. The load average was 9 to 12 (other lanes building), so the window was the quietest of the night but not quiet; the A/A spread of the copies is under 14% on reads, under 20% on `search --page 5`. `card update` restores pimdir.db and runs `sync` before each run, and its A/A stays wide (16 to 145%), its time bound by the disk. Scripts and JSON: ~/code/pimalaya/.lanes/cardamum (measure.sh, measure-update.sh, summary.sh, hf/h5-*).

| Command | 10k old | 10k rc | 100k old | 100k rc |
| --- | --- | --- | --- | --- |
| `card list -k home/contacts` | 7.20 ms | 2.49 ms | 8.97 ms | 2.59 ms |
| `card list -k home/contacts --page 200` | 26.26 ms | 9.26 ms | 37.62 ms | 15.52 ms |
| `card read -k home/contacts 3153` | 4.89 ms | 2.24 ms | 4.60 ms | 2.12 ms |
| `pimdir search alice` | 38.70 ms | 7.50 ms | 399.28 ms | 30.22 ms |
| `pimdir search --page 5 alice` | 154.10 ms | 16.39 ms | 1,912.75 ms | 132.55 ms |
| `pimdir count alice` | 27.86 ms | 4.00 ms | 346.53 ms | 3.50 ms |
| `card search alice` | 38.55 ms | 7.32 ms | 377.03 ms | 29.27 ms |
| `pimdir status` | 32.18 ms | 11.46 ms | 321.39 ms | 131.59 ms |
| `card update --if-match` | 9.05 ms | 7.04 ms | 14.81 ms | 7.19 ms |
| `card search alice` over the vdir (6,965 cards) | 77.33 ms | 60.77 ms | | |

Every rc median is at or under old; every page-1 read at 100k is under 50 ms (H3). The two regressions of I10 (`--page 200`, `pimdir status`) were app-side: both walked `list_summaries`, whose summary and address joins were 92% of the status (callgrind); they now read `list_items_page_asc` for the rows they only skip or count.
