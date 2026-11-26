---
cairn: change
id: follows-storage-draft-17
part: report
---

# Report

cardamum is on io-pimdir 0.20.0: a `[patch.crates-io]` git pin points at the release candidate 17d4ea8, the requirement already `version = "0.20.0"`. Gate G is green: fmt and the 11 clippy sets clean, 375 tests passed, 0 failed, 19 ignored, the CI steps 157 passed. `cargo deny` clean, `nix build .#default` succeeds. Nothing loses data, and no finding is open.

## Landed

- A, build: url 2.5.9, then the io-pimdir pin and its re-pins (7f16b27, cc3021b, 2314084, 17d4ea8). Features split by domain: only the pimdir backend links SQLite, and the binary no longer links ical-rs or mime-rs.
- B: the 15 compile breaks.
- C: a store an earlier draft wrote is refused `store-version`, in io-pimdir's words.
- E, search-draft-17, pinned by tests: `--sort title` with `name` a hidden alias; the reserved forms `~`, `/` and `{` refused; `label:` and `property:` answered on pimdir and refused elsewhere; `mid:` and the new date forms accepted; `set-property` rows listed, shown and cancelled like any row; the vdir matcher answers what the index answers, by title order and by relevance set.
- G1: an undecodable queue row is an item error named by its handle, the other rows listed (io-pimdir N1, answered in drop 2).
- G2: a refused query prints `Error:` once.
- H4 and the status and list fix: `pimdir search --page <n>` walks earlier pages on one index without reading their cards; `card list` and `pimdir status` read the rows they skip or count without summaries, removing the only two regressions (`card list --page 200`, `pimdir status`).
- I, docs: spec fold, CHANGELOG, README start-over note, the log entry cairn/log/2026-11-23-follows-storage-draft-17.md.
- J1, cross-app: cardamum reads neverest's draft-17 store (list, search, read, status, queue); a `card add` on a copy of that store queues a row the queue list shows.

## Final timings (inventory I11)

No window was quiet (load average 9 to 12); three copies of each binary agreed within 14% on reads.

| Command | 0.19.1, 10k cards | Now, 10k | 0.19.1, 100k cards | Now, 100k |
| --- | --- | --- | --- | --- |
| `card list` page 1 | 7.20 ms | 2.49 ms | 8.97 ms | 2.59 ms |
| `card list --page 200` | 26.26 ms | 9.26 ms | 37.62 ms | 15.52 ms |
| `card read` | 4.89 ms | 2.24 ms | 4.60 ms | 2.12 ms |
| `pimdir search alice` | 38.70 ms | 7.50 ms | 399.28 ms | 30.22 ms |
| `pimdir search --page 5 alice` | 154.10 ms | 16.39 ms | 1,912.75 ms | 132.55 ms |
| `pimdir count alice` | 27.86 ms | 4.00 ms | 346.53 ms | 3.50 ms |
| `card search alice` | 38.55 ms | 7.32 ms | 377.03 ms | 29.27 ms |
| `pimdir status` | 32.18 ms | 11.46 ms | 321.39 ms | 131.59 ms |
| `card update --if-match` | 9.05 ms | 7.04 ms | 14.81 ms | 7.19 ms |
| `card search alice` over the vdir | 77.33 ms | 60.77 ms | | |

Every page-1 read at 100k is under 50 ms, so acceptance H3 is met with no miss left. `card update` is bound by the disk (copies spread up to 145%), so its gain is within noise.

## Left

- J1, drain half: neverest pushing cardamum's queued `add` to Radicale was not run (the shared store was kept for himalaya's drain); the row has the shape neverest drains for himalaya.
- J2: the live suites need the owner's grant.
- K1 to K3, in order: io-pimdir 0.20.0 published by its tag workflow; every CLI on it; the git patch replaced by the released version, gate G, then cardamum's own release.
- Still wanted from io-pimdir: N2, `#[non_exhaustive]` on `PimdirAction`; N4, a count of a collection's items and bodied items (the status walk is still linear). N3 no longer matters to cardamum.

## Decisions for the owner (defaults taken)

- D1: features split as above.
- D2: `--sort title`, `name` a hidden alias (as amended by the lead).
- D3: no label or property command and no output key; the query fields only.
- D4: `set-property` rows listed like any other.
- D5: no `property-invalid` relay.
- D6: no migration; the start-over note only.
- D7: the url update in its own commit.
- From drop 2: an undecodable pending row is an item error rather than a listed row, and `pimdir status` counts it as no queued creation.
