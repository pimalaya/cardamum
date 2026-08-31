---
cairn: log
change: camelcase-json-keys
landed: 2026-08-29
---

# The JSON output keys are camelCase

The family is settling its `--json` output on one key convention, and Cardamum was emitting two. Its output types carried no `rename_all` at all, so serde printed the Rust field names as written, and a People listing answered a snake_case `next_page_token` next to the passthrough `nextPageToken` in the same document. camelCase is what the wire formats these backends wrap already speak (JMAP, Microsoft Graph, Google People), and unlike a hyphenated key it needs no `."key-name"` quoting in a `jq` path. Cardamum is 0.2.0, so the switch was made in one go with no compatibility alias.

**The attribute went on the output types only** (24 files, 39 types): every type whose value reaches `printer.out`, nested row and payload types included, now carries `#[serde(rename_all = "camelCase")]`. Nine keys changed spelling: `addressbookId` and `fnValue` on the card listing, `keptProperties` on the card update, `addressbookHomeSet` on CardDAV discovery, `displayName` and `syncToken` on the PROPFIND rows, `syncToken` on both sync reports, `dataBytes` on the raw REPORT rows, and `newState` with `hasMoreChanges` on the two JMAP changes commands. Every other key was already one word.

**Three shapes were left without the attribute**, none of them having a field to rename: the nine transparent newtypes wrapping a provider object (`JmapContactCardOutput`, `PeoplePersonOutput`, `MsgraphProfileGetOutput`, `RawJsonOutput` and their siblings), which serialize as the object itself; and the untagged `CarddavPropfindOutput`, whose two struct variants carry it instead. Adding it there would have been a no-op the next reader has to work out.

**The provider passthroughs kept their spelling**: `@odata.nextLink`, `@odata.deltaLink`, `nextPageToken`, `nextSyncToken`, `contactGroups`, and the JMAP `list`. A container `rename_all` does not reach a field carrying its own `rename`, which the regenerated schemas confirm rather than the documentation promising it: all six read back unchanged.

**The configuration was not touched.** The 49 `rename_all = "kebab-case"` attributes in config.rs are the TOML vocabulary, a separate one, and no type serves both paths. The four domain types in shared/addressbook/types.rs and shared/card/types.rs also spell kebab-case and were left alone: they are converted into row types before printing and never reach the printer themselves. The io-pimdir `v: 1` summary is the store's format, shared with Neverest, and is likewise none of this.

**The docs followed the keys**: eight command doc comments enumerate their output shape, and a doc comment is the help page, so `card list --help` now reads `{"cards": [{"id", "addressbookId", ...}]}`. The CHANGELOG entry for the four write commands that landed earlier this cycle spelled `kept_properties` and was corrected in place.

Verified: 57 schemas regenerated, zero snake_case properties across all of them and the nine new spellings present; `cargo fmt`, `check`, `clippy --all-targets` and 60 tests clean under `--all-features`; and the per-backend sweep green on each of the six backends alone, 17 to 33 tests each. No test asserted a JSON key, so none needed adjusting.

Spec updated: `commands` (ADDED: "JSON output keys are camelCase").
