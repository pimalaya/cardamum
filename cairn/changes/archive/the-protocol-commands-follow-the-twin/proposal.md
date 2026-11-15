---
cairn: change
id: the-protocol-commands-follow-the-twin
status: landed
created: 2026-11-15
---

# The protocol commands follow the twin

> Line C36 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), for X3. calendula took cardamum's namespaces in K6 (calendula c414b21, its change the-namespaces-follow-the-twin) and listed four points where cardamum, the model, was the one off the family.

## Why

- `carddav mkcol` shows `create` as a visible alias. The family hides a former name (calendula's `caldav create` for `caldav mkcalendar`, cardamum's own `card create` for `card add`): a visible alias is a second name for one command in the help and the completions.
- `carddav discover` prints bare `server: …` lines, the one record of cardamum not printed through pimalaya-cli's `Fields`; calendula's `caldav discover` prints `Server`, `Principal` and `Calendar home-set` as fields.
- The Graph commands naming a contact folder (`msgraph contact-folders get`, `child-folders`, `rename`, `delete`, and `msgraph contacts list`, `create`, `delta` under `-f`) fail uncoded when Graph holds no such folder, where the unified commands code it `addressbook-not-found` and calendula codes its Graph calendar commands `calendar-not-found`.
- `carddav propfind <ADDRESSBOOK>` says in its help that it sends "the lightweight `getetag` PROPFIND", but io-webdav enumerates a collection through an `addressbook-query` REPORT asking `getetag` alone; calendula's `caldav propfind` says so of its `calendar-query`.

## What

- src/carddav/mkcol.rs: `create` a hidden alias. Not breaking: `carddav create` still runs `carddav mkcol`, aliases are no contract, and its schema was always `cardamum-carddav-mkcol`.
- src/carddav/discover.rs: the text through `Fields`, `Server`, `Principal` and `Addressbook home-set`, `(unresolved)` for an endpoint not resolved, each value sanitized; the JSON unchanged. Breaking for a script cutting the text: MIGRATION.md.
- src/msgraph/client.rs `folder_refusal`, as `contact_refusal`: a 404 or a 400 `ErrorInvalidIdMalformed` coded `addressbook-not-found`, Graph's message kept, as calendula's `calendar_refusal`; another refusal keeps its own code. Applied to the seven commands. A code added breaks nobody (D12).
- src/carddav/propfind.rs: the help names the request each form sends, and its JSON output adds `truncated`, which the output carries.
- Spec: commands ("A former command name stays hidden", "carddav propfind lists the home set or one address book", added; "Text output is plain text" and the `addressbook-not-found` row of "A JSON error carries a stable code", modified), backends (the `addressbook-not-found` row of "A server's refusal carries its code", modified).
