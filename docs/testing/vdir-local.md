# vdir (local filesystem) — shared + specific command test report

- cardamum: `v0.2.0 --all-features` (io-vdir `0.1`)
- account: a throwaway `[accounts.vdir]` in a temp config (`--config /tmp/…`, never `~/.cardamumrc`) with `vdir.home-dir = /tmp/cardamum-vdir-test`, `addressbook.default = contacts`
- date: 2026-07-18
- method: a **fake local instance** under `/tmp` — no live provider, no golden-rule concern. Exercised the **shared** API (`addressbook`, `card`) *and* the **vdir-specific** subcommands (`vdir create/rename/delete/list`), cross-checking the two against each other and against the on-disk layout. The whole instance + temp configs were deleted at the end.

## Results — shared API

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | home present, home missing, `-b vdir`, `-b carddav` | ✅ `vdir: OK`; missing home → `FAIL (… does not exist)`; foreign `-b` bails |
| `addressbook list` | base, `--json`, `abook ls` | ✅ collections as addressbooks (id = dir name) |
| `addressbook create` | base, `-d`, `-C` | ✅ creates the collection dir + metadata files |
| `addressbook update` | `-n`, `-d` (set), `-d ""` (clear) | ✅ setting works; **clearing is a silent no-op (V1)** |
| `addressbook delete` | `-k`, missing `-k` | ✅ removes the dir; missing `-k` → clean clap error (exit 2) |
| `card create` | inline, stdin `-`, file, default `-k` | ✅ stores `<uuid>.vcf`; id = uuid (no `.vcf`) |
| `card read` | `<id>`, bad id | ✅ byte-faithful; bad id → `item … not found` |
| `card list` | base, `-s`/`-p` paging, `--json`, `cards ls` | ✅ all pass; paging splits client-side |
| `card update` | `<id>`, `--if-match` | ✅ overwrites; `--if-match` **silently ignored (V2)** |
| `card delete` | `<id>`, `card rm` | ✅ file removed |

## Results — vdir-specific API (`VdirCommand`)

| Command | Variants tested | Result |
| --- | --- | --- |
| `vdir create` | base, `-d`, `-C` | ✅ dir + `displayname` / `description` / `color` files written |
| `vdir list` | base, `--json` | ✅ id / display_name / description / color / path |
| `vdir rename` | existing, nonexistent | ✅ dir renamed, metadata preserved; nonexistent → friendly "not found" (**V4 fixed**) |
| `vdir delete` | existing, nonexistent | ✅ dir removed; nonexistent → friendly "not found" (**V4 fixed**) |

### `vdir item` subcommand (added 2026-07-18, iteration 1)

| Command | Variants tested | Result |
| --- | --- | --- |
| `vdir item create` | vcard (sniffed → `.vcf`), ical (`BEGIN:VCALENDAR` sniffed → `.ics`), `--kind ical` override, stdin, `item new` | ✅ correct extension per kind |
| `vdir item list` | base, `--json`, `items ls` | ✅ ID / **KIND** / SIZE / PATH; lists **both** vcard and ical |
| `vdir item get` | vcard, ical, `--json`, missing id | ✅ byte-faithful; missing id → clean error |
| `vdir item update` | ical (content change) | ✅ overwrites, **kind/extension preserved** (`.ics` stays `.ics`) |
| `vdir item delete` | id, `item rm` | ✅ file removed |
| missing collection | `item list nope` | ✅ friendly "Collection `nope` not found" (V4 path) |

**Item value-add proven:** in a collection holding 1 vCard + 2 iCalendar items,
`vdir item list` shows **all 3 with their kind**, while shared `card list` shows
**only the 1 vCard** — the specific API reaches items the shared API filters out.

**Interop:** a collection created via `vdir create` is immediately usable by shared `card create -k <it>` (card lands on disk, shared `card list` sees it), and a shared `addressbook create` collection shows up in `vdir list`. The shared and specific APIs operate on the same on-disk collections.

## Findings

### Bugs / issues

- **None.** Both API surfaces work end-to-end against the filesystem.

### Backend behaviour (not bugs)

- **V1 — clearing collection metadata is a silent no-op.** `addressbook update -k <id> -d ""` (or `-C ""`) reports success but leaves the `description` / `color` file in place: io-vdir's `update_collection` only *writes* non-empty metadata, it never removes a file. Setting a new value works; clearing does not. (Known io-vdir limitation.)
- **V2 — vdir has no ETag, so `card update --if-match` is silently ignored** (the backend documents `if_match` as ignored). Note the inconsistency with msgraph/jmap, which *bail* on `--if-match` rather than ignore it — vdir accepts and drops it. Defensible (a local store is last-write-wins), but a bail would be more honest.
- **V3 — byte-faithful storage.** Unlike the API backends (which convert vCard ↔ JSContact / provider models), vdir writes the **raw vCard verbatim** to `<uuid>.vcf`, so the round-trip is byte-exact and the client `UID` is preserved untouched. The card id is the item uuid *without* the `.vcf` suffix; the file on disk is `<id>.vcf`, and the id round-trips through read/update/delete.
- **V4 — raw OS errors on a missing collection — FIXED.** `vdir rename/delete nope` (and every `vdir item` command) previously surfaced `No such file or directory (os error 2)`. Iteration 1 added `VdirClient::collection_path`, which validates the directory and bails "Collection `<name>` not found".

### Observations

- On-disk model: each immediate subdirectory of `home-dir` is a collection (addressbook); collection metadata rides in `displayname` / `description` / `color` files; cards are `<uuid>.vcf` files. Matches the vdir spec.
- No live-provider risk: everything ran in a throwaway `/tmp` instance, fully removed afterward.

## Verdict

The vdir backend is **fully working across both the shared API and its own `vdir` subcommands** — now including the **`item` subcommand** (iteration 1 of the specific-API build-out), which reaches raw items of any kind (vCard *and* iCalendar) that the shared `card` API filters out. The two APIs interoperate on the same on-disk collections. No bugs; **V4 is fixed** (friendly "collection not found"). Two caveats remain: **V1** (clearing description/color is a silent no-op, an io-vdir limitation) and **V2** (`card update --if-match` is silently ignored rather than bailed as on msgraph/jmap). Next in the specific-API round: CardDAV (flat WebDAV-method list).
