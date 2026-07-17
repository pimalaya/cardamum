# CardDAV on iCloud — shared-command test report

- cardamum: `v0.2.0 --all-features` (io-webdav git `f2376eb` with the verbatim card-id fix; pimalaya-config git with the wizard `toml::to_string` serializer)
- account: `icloud` (`carddav.discover = "icloud.com"`, HTTP Basic with an app-specific password)
- date: 2026-07-17
- method: connection + card CRUD validated by hand. iCloud forbids creating address books (`403`), so per the [golden rule](provider-test-plan.md) the card operations ran on a single uniquely-marked throwaway contact inside the existing `card` book, addressed only by the id `create` returned and deleted afterwards. The account's real contacts were never listed or modified.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base (discover route: PACC / RFC 6764 + app password) | ✅ `carddav: OK` |
| `addressbook list` | base, `--json` | ✅ one book, `card` |
| `addressbook create` | base | ⛔ `403` — iCloud forbids new address books (expected, not a bug) |
| `card create` | stdin `-`, `-k card` | ✅ with a vCard **3.0 + `N`** (see P1); id is `<uuid>.vcf` |
| `card read` | `<id>` | ✅ pass (**F1 fixed**) |
| `card update` | `<id>` | ✅ update lands — read-back confirms (**F2 fixed**) |
| `card delete` | `<id>` | ✅ removed — read-after-delete `404`s (**F2 fixed**) |

Not exercised on this real account: `card list` / paging (would enumerate the account's real contacts — skipped for privacy); `addressbook update` / `delete` (the sole `card` book is real).

## Findings

### Bugs / issues

- None new. The F1/F2 blockers (the io-webdav `.vcf` path asymmetry) are fixed — re-validated here on a **second** `.vcf`-suffixing server: `card read` returns the card, `card update` lands (read-back), `card delete` removes it (`404` after). The card id is the `.vcf`-suffixed href verbatim, exactly as on Fastmail.

### Provider-specific behaviour (not bugs)

- **P1 — iCloud requires a vCard 3.0 with an `N` property.** A `VERSION:4.0` card is rejected with `403 VCARD parse error`; a `3.0` card carrying `N:…` + `FN` is accepted. Fastmail accepted the 4.0 card and down-converted, but iCloud validates the input strictly. cardamum forwards the body unchanged (it does not parse vCards), but the rejection is now legible: a `403 valid-address-data`/`VCARD` write surfaces an actionable hint plus the server's response (here `VCARD parse error`) instead of a raw dump. It compounds with the Fastmail report's F3 (both servers require a `UID`).
- **P2 — iCloud forbids creating address books over CardDAV** (`403` on MKCOL), and exposes a single fixed `card` book. Any test there must use the throwaway-contact fallback.

## Verdict

CardDAV on iCloud works for the card surface (create / read / update / delete) with the io-webdav verbatim-id fix, and the discovery route (PACC / RFC 6764 + app-specific password) connects cleanly. The fix is now confirmed on two independent `.vcf`-suffixing servers (Fastmail, iCloud). The only things to be aware of are provider behaviours, not cardamum bugs: iCloud's strict vCard input (3.0 + `N`, P1) and its read-only address-book set (P2). The cross-provider follow-up remains cardamum's F3 — surfacing a friendlier error (or normalising the vCard) when the input a user passes to `card create` is rejected by a strict server.
