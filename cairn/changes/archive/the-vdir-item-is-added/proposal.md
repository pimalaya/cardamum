---
cairn: change
id: the-vdir-item-is-added
status: landed
created: 2026-11-16
---

# The vdir item is added

## Why

pimalaya-cli's commands.md ("A verb keeps one meaning") has `add` create an item and `create` a collection. `vdir item create` made an item, where calendula and Himalaya's local namespaces take `add`. The owner's ruling of 2026-11-16 (D32, line L18 of the CLI alignment round) aligns the local namespaces on `add`; the protocol namespaces keep the protocol's own verb.

## What

- `vdir item create` becomes `vdir item add`, its types and file following (`VdirItemAddCommand`, `VdirItemAddOutput`, src/vdir/item/add.rs); `new` stays its visible alias and `create` becomes a hidden one, so a script keeps working.
- Its `json-schema` key becomes `cardamum-vdir-item-add`. `cardamum-vdir-item-create` shipped in v0.3.0, so the change is breaking: a CHANGELOG **Breaking** line and a MIGRATION.md before/after entry.
- `vdir create`, a collection, stays. The text and JSON output are unchanged. No MCP tool reaches `vdir` (src/mcp.rs `TOOLS` lists the unified API alone), so no tool name moves.
