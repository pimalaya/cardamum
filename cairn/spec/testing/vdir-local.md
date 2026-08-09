# vdir (local filesystem): shared and specific command test report

- cardamum: v0.2.0 `--all-features` (rev `42ca366`, working tree; io-vdir 0.1)
- account: a throwaway `[accounts.vdir]` in a temp config (`--config <scratch>/vdir.toml`, never `~/.cardamumrc`) with `vdir.home-dir` under the scratch directory and `addressbook.default = contacts`
- date: 2026-08-09 (re-run; first run 2026-07-18)
- method: a **fake local instance**, so no live provider and no golden-rule concern. Exercised the shared API (`addressbook`, `card`) and the vdir-specific subcommands (`vdir create/rename/delete/list` plus `vdir item …`), cross-checking the two against each other and against the on-disk layout, then deleted the whole instance and both temp configs.

## Results: shared API

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | home present, home missing, `-b vdir`, `-b carddav` | ✅ `vdir: OK`; a missing home reports `FAIL (… does not exist)`, though the command still **exits 0 (V9)**; unconfigured `-b` bails |
| `addressbook list` | base, `--json`, `abook ls` | ✅ collections as addressbooks, id = directory name |
| `addressbook create` | base, `-d`, `-C` | ✅ creates the directory plus the `displayname` / `description` / `color` files |
| `addressbook update` | `-n`, `-d`, `-C` (set), `-d ""` / `-C ""` (clear), no field flag, unknown id | ✅ setting works, a flagless update bails, an unknown id → "Addressbook `nope` not found", and clearing removes the metadata file (**V1 fixed**) |
| `addressbook delete` | `-k`, missing `-k`, unknown id | ✅ removes the directory; missing `-k` → clap error, exit 2; an unknown id → "Addressbook `nope` not found" (**V8 fixed**) |
| `card create` | file, stdin `-`, default `-k`, **unknown `-k`** | ✅ stores `<uuid>.vcf`, id = the uuid without the extension; an unknown collection now fails and creates nothing (**V6 fixed**) |
| `card read` | `<id>`, `--json`, bad id | ✅ byte-faithful (V3); a bad id → "item … not found" |
| `card list` | base, `-s`/`-p` paging, `--json`, `cards ls`, default `-k`, **unknown `-k`** | ✅ listing and paging; an unknown collection → "Addressbook `nope` not found" (**V7 fixed**) |
| `card update` | `<id>`, `--if-match`, **absent id** | ✅ overwrites byte-faithfully; `--if-match` is refused (**V2 fixed**); an absent id fails and creates nothing (**V5 fixed**) |
| `card delete` | `<id>`, `card rm`, absent id | ✅ file removed; an absent id → "item … not found" |

## Results: specific API (`vdir …`)

| Command | Variants tested | Result |
| --- | --- | --- |
| `vdir create` | base, `-d`, `-C` | ✅ directory plus metadata files written |
| `vdir list` | base, `--json` | ✅ id / display name / description / color / path |
| `vdir rename` | existing, unknown | ✅ directory renamed, metadata preserved; unknown → "Collection `nope` not found" |
| `vdir delete` | existing, unknown | ✅ directory removed; unknown → "Collection `nope` not found" |
| `vdir item create` | vCard (sniffed → `.vcf`), iCalendar (sniffed → `.ics`), stdin | ✅ the right extension per kind |
| `vdir item list` | base, `--json`, unknown collection | ✅ ID / KIND / SIZE / PATH, listing **both** kinds; unknown → "Collection `nope` not found" |
| `vdir item get` | vCard, iCalendar | ✅ byte-faithful |
| `vdir item update` | content change, **absent id** | ✅ overwrites and keeps the kind (`.ics` stays `.ics`); **an absent id fails properly**, unlike the shared `card update` (V5) |
| `vdir item copy` / `move` | a → b copy, a → b move | ✅ copy leaves both, move drops the source |
| `vdir item delete` | `<id>` | ✅ file removed |

**Item value-add, re-confirmed:** in a collection holding one vCard and one iCalendar item, `vdir item list` shows both with their kind while the shared `card list` shows only the vCard.

**Interop, re-confirmed:** a collection created with `vdir create` is immediately usable by the shared `card create -k`, and a card written through the shared API appears in `vdir item list`. Both APIs operate on the same directories.

## Findings

### Bugs / issues

The specific surface is clean. Every defect below was on the **shared** arm, each the local twin of something already fixed on a network backend today, and all six are now fixed too (see the [vdir-write-guards](../../log/2026-08-09-vdir-write-guards.md) log entry).

- **V5: `card update <absent-id>` created the file: FIXED.** Exactly the CardDAV F5 defect, fixed there this morning by reading the card first: on vdir the shared update called `store_item` with whatever id it was given, so `card update -k contacts absent-id …` wrote `absent-id.vcf` and reported success, while the specific `vdir item update` on the same id failed properly. The shared update now locates the item first, and an unknown id fails on that lookup with no file created.
- **V6: `card create -k <unknown>` created the collection: FIXED.** A typo in `-k` invented an addressbook and hid the card inside it, where every other backend refuses to write into a book that does not exist (Fastmail 404, Graph 400, People 404). The shared path resolver now checks the directory, so the write fails with "Addressbook `x` not found" and nothing is created.
- **V7: `card list -k <unknown>` printed an empty table and exited 0: FIXED.** An unknown addressbook was indistinguishable from an empty one, where every other backend errors and the specific `vdir item list` names the missing collection. Same resolver, same message.
- **V8: `addressbook delete -k <unknown>` surfaced a raw OS error: FIXED.** It leaked `No such file or directory (os error 2)`, the V4 fix from July having given only the specific commands a friendly message through `VdirClient::collection_path`. The shared arm now does the same check.
- **V1 (carried over from July): clearing collection metadata was a silent no-op: FIXED.** `addressbook update -k <id> -d ""` reported success and left the `description` file in place, the vdir twin of the CardDAV F4 that io-webdav fixed this morning with `DAV:remove`: io-vdir wrote a file per present non-empty value and had no removal path. Its collection update now writes the collection it is handed, absent or empty values having their file removed, which is what taking a whole `VdirCollection` rather than a patch already promised. Re-verified: clearing removes the files and the values come back when set again.
- **V2 (carried over from July): `card update --if-match` was silently ignored: FIXED.** vdir has no ETag, which is fine, but msgraph and jmap bail on the same flag rather than accept and drop it, and accepting a guard nobody honours is the least honest of the three options. It bails now, and the backends spec gained the requirement.

### Backend behaviour (not bugs)

- **V3: storage is byte-faithful.** Unlike the API backends, vdir writes the raw vCard verbatim, so a read is byte-identical to what was written (verified with `diff`) and the client `UID` survives untouched. The card id is the file stem, and the file is `<id>.vcf`.
- **V4 (fixed in July, still holding): a missing collection reports "Collection `<name>` not found"** on every specific command, including the whole `item` subcommand.
- On-disk model: each immediate subdirectory of `home-dir` is a collection, metadata rides in `displayname` / `description` / `color` files, and items are `<uuid>.<vcf|ics>`. Matches the vdir spec.

### Observations

- **V9: `account check` exits 0 even when a backend reports FAIL.** The report says `vdir: FAIL (…)` and `--json` carries `"ok": false`, but the exit code is 0, so a script cannot tell a healthy account from a broken one without parsing the output. Not vdir-specific: the command reports per-backend status and returns success regardless.
- No live-provider risk: the whole instance lived in the scratch directory and was removed afterwards, along with both temp configs.

## Verdict

The vdir-specific surface is in good shape: collections, the kind-aware `item` subcommand (vCard and iCalendar), copy and move, byte-faithful get and update, and friendly errors for a missing collection throughout. The shared arm was where the work was: **six defects**, four of them new (V5 to V8) and all of the same family this round has been clearing on the network backends, plus the two carried over from July (V1, V2). The pattern was consistent, and telling: the shared commands wrote through to the filesystem without first asking whether the thing they addressed exists, so an unknown collection was created, an absent card was created, an unknown collection listed as empty, and a raw OS error escaped. All six are fixed and re-verified, the shared arm now doing the same check the specific one has done since July; see the [vdir-write-guards](../../log/2026-08-09-vdir-write-guards.md) log entry. **V9** stands: `account check` still exits 0 when a backend reports FAIL, which is not vdir's to fix.
