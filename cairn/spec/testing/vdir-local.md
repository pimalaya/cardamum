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
| `addressbook update` | `-n`, `-d`, `-C` (set), `-d ""` / `-C ""` (clear), no field flag, unknown id | ✅ setting works and a flagless update bails; an unknown id → "Addressbook `nope` not found"; **clearing is still a silent no-op (V1)** |
| `addressbook delete` | `-k`, missing `-k`, unknown id | ✅ removes the directory; missing `-k` → clap error, exit 2; **an unknown id surfaces a raw OS error (V8)** |
| `card create` | file, stdin `-`, default `-k`, **unknown `-k`** | ✅ stores `<uuid>.vcf`, id = the uuid without the extension; **an unknown collection is created on the spot (V6)** |
| `card read` | `<id>`, `--json`, bad id | ✅ byte-faithful (V3); a bad id → "item … not found" |
| `card list` | base, `-s`/`-p` paging, `--json`, `cards ls`, default `-k`, **unknown `-k`** | ✅ listing and paging; **an unknown collection lists as empty, exit 0 (V7)** |
| `card update` | `<id>`, `--if-match`, **absent id** | ✅ overwrites byte-faithfully; **`--if-match` is silently ignored (V2)**; **an absent id creates the file (V5)** |
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

The specific surface is clean. Every defect below is on the **shared** arm, and each one is the local twin of something already fixed on a network backend today.

- **V5 (new): `card update <absent-id>` creates the file.** Exactly the CardDAV F5 defect, fixed there this morning by reading the card first: on vdir the shared update calls `store_item` with whatever id it is given, so `card update -k contacts absent-id …` writes `absent-id.vcf` and reports success. The specific `vdir item update` on the same absent id fails properly, so the shared path is the outlier.
- **V6 (new): `card create -k <unknown>` creates the collection.** A typo in `-k` invents an addressbook and hides the card inside it. Every other backend refuses to write into a book that does not exist (Fastmail 404, Graph 400, People 404).
- **V7 (new): `card list -k <unknown>` prints an empty table and exits 0**, so an unknown addressbook is indistinguishable from an empty one. Every other backend errors, and the specific `vdir item list` names the missing collection.
- **V8 (new): `addressbook delete -k <unknown>` surfaces a raw OS error**, `No such file or directory (os error 2)`. The V4 fix from July gave the specific commands a friendly "Collection `<name>` not found" through `VdirClient::collection_path`; the shared delete never got it.
- **V1 (still open): clearing collection metadata is a silent no-op.** `addressbook update -k <id> -d ""` reports success and leaves the `description` file in place. This is the vdir twin of the CardDAV F4, fixed today in io-webdav by emitting `DAV:remove`: io-vdir's collection update writes non-empty metadata and never removes a file.
- **V2 (still open): `card update --if-match` is silently ignored.** vdir has no ETag, and the backend documents the argument as ignored, but msgraph and jmap bail on the same flag rather than accept and drop it. Accepting a guard nobody honours is the least honest of the three options.

### Backend behaviour (not bugs)

- **V3: storage is byte-faithful.** Unlike the API backends, vdir writes the raw vCard verbatim, so a read is byte-identical to what was written (verified with `diff`) and the client `UID` survives untouched. The card id is the file stem, and the file is `<id>.vcf`.
- **V4 (fixed in July, still holding): a missing collection reports "Collection `<name>` not found"** on every specific command, including the whole `item` subcommand.
- On-disk model: each immediate subdirectory of `home-dir` is a collection, metadata rides in `displayname` / `description` / `color` files, and items are `<uuid>.<vcf|ics>`. Matches the vdir spec.

### Observations

- **V9: `account check` exits 0 even when a backend reports FAIL.** The report says `vdir: FAIL (…)` and `--json` carries `"ok": false`, but the exit code is 0, so a script cannot tell a healthy account from a broken one without parsing the output. Not vdir-specific: the command reports per-backend status and returns success regardless.
- No live-provider risk: the whole instance lived in the scratch directory and was removed afterwards, along with both temp configs.

## Verdict

The vdir-specific surface is in good shape: collections, the kind-aware `item` subcommand (vCard and iCalendar), copy and move, byte-faithful get and update, and friendly errors for a missing collection throughout. The shared arm is where the work is: **six defects**, four of them new (V5 to V8) and all of the same family this round has been clearing on the network backends, plus the two carried over from July (V1, V2). The pattern is consistent, and telling: on vdir the shared commands write through to the filesystem without first asking whether the thing they are addressing exists, so an unknown collection is created, an absent card is created, an unknown collection lists as empty, and a raw OS error escapes. None of it is deep, and the specific arm already has the validation the shared arm needs (`VdirClient::collection_path`).
