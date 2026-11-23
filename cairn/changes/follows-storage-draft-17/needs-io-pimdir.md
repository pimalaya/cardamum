---
cairn: change
id: follows-storage-draft-17
status: active
created: 2026-11-23
---

# What cardamum needs from io-pimdir

For the io-pimdir lane (io-pimdir cairn/changes/what-the-consumers-need). Nothing here blocks the migration: cardamum builds, passes its suite and clippy on 7f16b27 with the fixes of tasks.md B (inventory.md, probe).

## N1. `pending_actions` lists a row it cannot decode

- **What**: `PimdirReader::pending_actions` returns each pending row of a collection, a row whose payload does not decode as an entry of its own (its id, kind and the decode error), instead of failing the whole collection.
- **Why**: `pimdir queue list` (src/pimdir/backend.rs `pending_rows`) can then list the valid rows of that address book and name the bad one by its row and handle. Today one undecodable row hides every pending row of its address book behind one `action-malformed` item error with `id` `null` (cardamum cairn/log/2026-11-20-the-contract-audit-rulings.md).
- **Evidence**: unchanged at 7f16b27 (same signature as v0.19.1); open in pimdir cairn/changes/the-freeze-plan/tasks.md:68 (finding G1) and in the contract audit's handoff (~/.claude/plans/pimalaya-family-ledgers/cli-contract-audit-2026-11-20/handoff-pimdir.md, item 8).
- **App test**: `an_undecodable_pending_row_is_an_item_error` (src/pimdir/backend.rs) changes to expect the valid rows listed and the item error naming the row.
- **Workaround**: the current behaviour, kept; task G1 is parked until a release carries it.
- **Status**: answered at io-pimdir 2314084 (2026-11-23): the row is listed as `PimdirAction::Unknown`, `codec::payload_outcome` judging it malformed. cardamum names it alone (task G1 done).

## N2. `PimdirAction` growth before 1.0

- **What**: `#[non_exhaustive]` on `PimdirAction` before io-pimdir 1.0, or the set stated closed.
- **Why**: `SetProperty` broke cardamum's exhaustive `match` in `row_target` (src/pimdir/backend.rs:370, compile break B2). A queue reader has to name every kind another program may queue.
- **Evidence**: pimdir cairn/changes/the-freeze-plan/tasks.md:67 (`non_exhaustive` on every enum expected to grow).
- **Workaround**: the `SetProperty` arm (B2); a wildcard arm once the enum is non-exhaustive.

## N3. A summary walk costs a quarter more per row

- **What**: `PimdirReader::list_summaries` (keyset batches of a collection with each item's summary) reads a row about 25 to 35% slower at 7f16b27 than at 0.19.1.
- **Why**: cardamum's two whole-collection walks regress: `card list --page 200` (the cards before the page counted off their rows, STORAGE order) and `pimdir status` (every card of every address book walked to count `total` and `hydrated`).
- **Evidence**: inventory I10, 10k-card stores from one generator seed at both drafts, release binaries, A/A copies within 4%: `card list -k home/contacts --page 200` 21.5 to 26.7 ms, `pimdir status` 26.0 to 34.7 ms (6,965 cards); every other read got faster. At 100k cards: `card list -k home/contacts --page 200` 40.3 to 45.7 ms, `pimdir status` 418 to 614 ms; the page-1 reads (list, read, search, count) 2.4 to 36 ms, down from 4.9 to 521 ms.
- **App test**: tasks H2's two measures; acceptance H3.
- **Workaround**: none needed for correctness; cardamum ships slower on these two until a release fixes it, or N4 lands.
- **Status**: worked around app-side on 2026-11-23: the rows `card list` skips and the rows `pimdir status` counts are read by `list_items_page_asc`, without summaries or addresses (callgrind: `list_summaries` 92% of `pimdir status`, `attach_addresses` 45%). At 100k cards: `card list --page 200` 46.5 to 16.8 ms, `pimdir status` 565 to 167 ms, both now under 0.19.1's. The per-row cost of `list_summaries` stays for io-pimdir to judge; cardamum no longer needs it.

## N4. Counts of a collection's items and bodied items

- **What**: a reader count of a collection's live items and of those holding a body, the pending overlay folded or not, as `count_pending_creates` counts creates.
- **Why**: `pimdir status` (src/pimdir/backend.rs:133-137) walks every summary to count two integers; audit-2026-11-12 C5 already notes "a count of bodied items would need a statement".
- **Evidence**: as N3; at 100k cards (seed 7, 1,000,000 items, 68,837 cards in the account `home`) the walk is linear in the store: `pimdir status` 418 ms at 0.19.1, 614 ms at cc3021b (+47%, A/A under 9%, measured 2026-11-23 while other lanes built).
- **App test**: `pimdir status` counts unchanged on the indexed fixtures (src/pimdir/backend.rs tests), its H2 measure.
- **Workaround**: the walk, kept, without summaries (see N3). Still wanted: the walk stays linear in the address book.

## Known defects the frame lists (pimdir cairn/changes/a-smaller-sync/report.md)

cardamum runs no engine and queues `add`, `update` and `remove` alone: K4 (a queued `copy` or `move`) is unreachable from its writes, and rekey (R1, R4) is neverest's. No app-level test is owed; a card whose conflict a rekey clears (R1) would read as an ordinary card in cardamum, nothing cardamum can detect.
