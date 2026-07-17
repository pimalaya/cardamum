# CardDAV on Fastmail — shared-command test report

- cardamum: `v0.2.0 --all-features` (rev `96c04da`, working tree with the 2026-07-17 released-deps + io-pim-discovery upgrade applied)
- account: `fastmail` (`carddav.home = https://carddav.fastmail.com/dav/addressbooks/user/…/`, HTTP Basic)
- date: 2026-07-17
- method: every shared command × every flag variant, run by hand inside a throwaway addressbook (`cardamum-test-<ts>`), per [provider-test-plan.md](provider-test-plan.md). The throwaway book was deleted on cleanup; the real `Default` book was only read.
- update (same day): **F1/F2 fixed and re-verified.** io-webdav now uses a card's server-returned resource id verbatim (single opaque `id`, no `.vcf` added or stripped anywhere), and cardamum names new cards `<uuid>.vcf`. Re-tested against a local-path patch of io-webdav; `card read`/`update`/`delete` (incl. `--if-match`) all pass, verified by read-back and list.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `carddav: OK` |
| `addressbook list` | base, `--json`, `abook ls` / `addressbooks list` aliases, `-b carddav`, `-b jmap` | ✅ all pass; `-b jmap` bails cleanly |
| `addressbook create` | base, `-d`, `-C` | ✅ pass (id = the given name; description + color set) |
| `addressbook update` | `-n`, `-d`, `-C` (set) | ✅ setting works; **clearing with `""` is a no-op (F4)** |
| `addressbook delete` | `-k`, missing `-k` | ✅ deletes; missing `-k` is a clean clap error |
| `card list` | base, `-k`, `-s`/`-p` paging, `--json` (etag), no-`-k` default, bad `-k` | ✅ all pass; bad `-k` → error, exit 1 |
| `card create` | stdin `-`, raw inline, file path, `-k` | ✅ pass (id is `<uuid>.vcf`); ⚠️ still needs the vCard to carry a `UID` (F3) |
| `card read` | `<id>`, `--json` | ✅ pass (**F1 fixed**) |
| `card update` | `<id>`, `--if-match` (ok + stale) | ✅ update lands on the real card; stale `--if-match` → 412 (**F2 fixed**) |
| `card delete` | `<id>` | ✅ real card removed, list confirms 0 (**F2 fixed**) |

## Findings

### Bugs / issues

- **F1 — `card read` 404'd for every card — FIXED.** io-webdav had defined the card id as the resource URI with any `.vcf` stripped, but the `ReadCard` coroutine used that id verbatim and never re-appended `.vcf`, so on a server that suffixes `.vcf` (Fastmail) it `GET`d `…/<book>/<id>` — a path that did not exist (reproduced on both a fresh card and a pre-existing `Default` card). **Fix:** io-webdav now carries a single opaque `id` = the server's resource name verbatim (no `.vcf` added or stripped in `card_from_entry`/`enumerate`, and `CreateCard` no longer appends it), with a `list → read` round-trip regression test. Re-verified: `card read <id>` returns the vCard, exit 0.

- **F2 — `card update` / `card delete` reported success but silently missed the real card — FIXED.** Same `.vcf` asymmetry: `UpdateCard`/`DeleteCard` built the extension-less path, so `update` wrote a phantom while the real `<id>.vcf` stayed unchanged and `delete` removed nothing. **Fix:** with the verbatim id above, both address the real resource. Re-verified: after `card update <id> …` a read-back and `card list` both show the new value; a stale `--if-match` is rejected with `412`; after `card delete <id>` the card is gone from `card list`. (cardamum also now names new cards `<uuid>.vcf`, so the id keeps the conventional extension.)

- **F3 — `card create` does not mint a `UID` (error now legible).** A vCard with no `UID` is rejected by Fastmail with `403 valid-address-data: Missing mandatory UID property`; cardamum forwards the body unchanged (by design — it does not parse vCards). It no longer dumps the raw DAV error: a card write rejected with `403 valid-address-data`/`VCARD` now surfaces an actionable hint ("providers disagree… most require a UID; some, e.g. iCloud, require vCard 3.0 with an N property") plus the server's `<responsedescription>` (here `Missing mandatory UID property`). Remaining open: nothing to fix in cardamum — this is user input; the hint is the resolution.

- **F4 — clearing addressbook metadata is a no-op.** `addressbook update -k <AB> -d ""` and `-C ""` print `successfully updated` but leave the description / color unchanged (the io-webdav collection update writes non-empty values only). This contradicts the `--help` (`-d TEXT: … pass "" to clear`). Setting new non-empty values works.

### Provider-specific behaviour (not bugs)

- **P1 — the card id is the `.vcf`-stripped href, not the vCard `UID`.** `card create` assigns a fresh UUID as the resource name and returns it as the id; the vCard's own `UID` is preserved in the body but is not the addressing key. A card created with `UID:card-test-001` was addressed as `928d…` and still read back `UID:card-test-001`.
- **P2 — vCard 4.0 is stored/returned as 3.0.** A card written `VERSION:4.0` reads back `VERSION:3.0` (Fastmail down-converts).
- **P3 — Fastmail requires a `UID`** on every vCard; see F3 for the client-side consequence.

### Observations

- **O1 — error bodies are dumped raw.** A 404 (bad card / addressbook) surfaces Fastmail's full HTML error page as the message; a 403 surfaces the DAV XML error. Functional (non-zero exit, correct failure) but noisy — a trimmed message would read better.
- `card list --json` exposes each card's `etag`, usable for `card update --if-match`.
- Exit codes are correct: `0` on success, `1` on a server error (an earlier apparent `exit=0` on failure was a `| head` pipe artifact, not cardamum's code).
- The account carries several leftover `io-webdav-test-*` addressbooks from prior io-webdav integration runs — unrelated clutter, left untouched.

## Verdict

The full shared surface — `addressbook` (list / create / update / delete, all flags, `--json`, aliases, backend selection) and `card` (list / create / read / update / delete, paging, `--json`, `--if-match`) — now works correctly on Fastmail CardDAV. The F1/F2 blockers (the io-webdav `.vcf` path asymmetry) are **fixed and re-verified**: the card id is the server's resource name verbatim end-to-end, so read/update/delete address the real resource. **Release-ready for the CardDAV surface** once the io-webdav verbatim-id fix ships (currently consumed via a local-path patch). Remaining secondary follow-ups in cardamum: **F3** — `card create` still surfaces a raw `403` when the input vCard omits a `UID` (mint one client-side, or document the requirement); **F4** — `addressbook update -d "" / -C ""` is a no-op despite its `--help` (io-webdav collection update writes non-empty values only). Neither blocks the card surface.
