---
cairn: log
change: a-card-is-added
date: 2026-11-15
---

# A card is added

`card add` creates a card, as Himalaya's `message add`, `card create` a hidden alias and `card new` still visible (decision D10). Task C6 of cli-alignment-2026-11-15 (finding F25), twin of calendula's K8. Capabilities **commands**, **mcp** and **backends** (wording) moved.

- **src/shared/card/add.rs** (was create.rs): `CardAddCommand`, `CardAddOutput`, `CardAddedOutput`; the REPL's composer refusal, the capability help and the MCP table follow.
- **MCP**: the tool is `card_add`. pimalaya-cli lists tools and refuses an unlisted name before the app sees the call, so no hidden `card_create` tool can stay; the server is new since v0.3.
- **json-schema**: `cardamum-card-add`, where v0.3 wrote `cardamum-card-create`.
- **Evidence**: tests/repl.rs (`card add -i` refused for the composer, the alias `card create -` parsed and refused `stdin-taken`), tests/mcp.rs (`card_add` listed, called with text and refused a path and stdin), the pimdir receipt tests parsing `add`.

CHANGELOG (Changed, the Unreleased lines on the final name), MIGRATION.md, README. Spec: commands ("A card is added" added; every requirement naming the verb), mcp ("Tools are the classified unified API"), backends (two mentions).
