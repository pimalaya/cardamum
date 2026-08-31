---
cairn: change
id: output-schemas-and-path-expansion
status: landed
created: 2026-08-29
---

# Every data command describes its JSON, and every path field expands where it is read

Group B of the cross-repository CLI alignment (CLI_ALIGNMENT_PLAN.md) brings Cardamum onto the family's config and output conventions. Four of its items apply here.

**A path field expanded at a call site is a path field the next call site forgets.** `pimdir.root` was declared a bare `PathBuf` and expanded inside `PimdirClient::new`, so the expansion was a property of one constructor rather than of the field. `tls.cert` had no expansion at all: a `cert = "~/ca.pem"` reached rustls as the literal relative path `./~/ca.pem`, and the connection failed naming a file nobody wrote. The spec already requires path fields to expand; both were violations of it, one visible and one not.

**A CardDAV server may ask for no credentials.** `CaldavAuthConfig` in Calendula already carries a `None` variant for that; `CarddavAuthConfig` did not, so an unauthenticated server could not be configured at all. io-webdav has spoken `WebdavAuth::None` all along.

**Nothing described the `--json` output.** Cardamum had zero `JsonSchema` derives and no `json-schema` command against 84 `printer.out` call sites, so a script consuming `--json` had only the prose in each command's `--help` to go on. The output types were also named after their rendering (`CardsTable`, `SyncReport`, `PersonReport`) rather than after the command they answer, so nothing tied a type to the invocation that prints it.

**comfy-table was reached directly.** pimalaya-cli re-exports it under `table` precisely so the toolkit owns that version; Calendula already goes through it.

## What changes

- Four commands whose confirmation carried data (`addressbook create`, `card create`, `card update`, `vdir item create`) answer an output type of their own, the identifier and the kept properties leaving the prose.
- `pimdir.root` gains `#[serde(deserialize_with = "shell_expanded_path")]` and `PimdirClient::new` drops its expansion. `tls.cert` gains `#[serde(default, deserialize_with = "opt_shell_expanded_path")]`, a private helper next to the config type carrying a TODO for the shared one pimalaya-config does not ship yet.
- `CarddavAuthConfig` gains a `None` variant, wired through `build_auth` and offered by the wizard when discovery advertised no scheme at all.
- Every command returning data gets a `<Domain><Target><Verb>Output` type deriving `Display`, `Serialize` and `JsonSchema`, registered in a new src/json_schema.rs, behind a new `json-schema` command aliased `json-schemas`. `Message` stays for confirmations only. A wire object with no `JsonSchema` of its own is described as raw JSON, the shape being the provider's rather than ours.
- Every `comfy_table` import becomes a `pimalaya_cli::table` one and the direct dependency goes away.

## What does not change

The JSON payloads of the renamed types. Every one keeps the field names it had, so a script reading `--json` today reads the same document tomorrow; only the Rust names moved. The four confirmations turning into output types are the exception, and the CHANGELOG marks them breaking.

The `carddav.server` shape stays `Option<String>` accepting a full URL, a bare domain or `domain:port`. That is the shape the family standardised on, and Calendula is moving onto it.

The wizard's own expansion of a typed folder path stays where it is: it validates an answer before it becomes a config field, and the expanded value is what gets written.
