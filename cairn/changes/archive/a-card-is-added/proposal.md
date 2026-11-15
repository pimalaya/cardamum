---
cairn: change
id: a-card-is-added
status: landed
created: 2026-11-15
---

# A card is added

## Why

Himalaya adds an item (`message add`) where cardamum created one (`card create`). Decision D10 rules `add` for an item, `create` a hidden alias. Finding F25, task C6 of cli-alignment-2026-11-15, twin of calendula's K8.

## What

- `card add`, `card create` a hidden alias, `card new` still visible; the capability was `card.add` already.
- The module and its types follow the verb: src/shared/card/add.rs, `CardAddCommand`, `CardAddOutput`, `CardAddedOutput`.
- The MCP tool is `card_add`. pimalaya-cli's `serve` refuses a tool name it did not list before the app sees the call (its src/mcp.rs `call`, "Unknown tool"), so `card_create` cannot stay as an unlisted alias; the MCP server is new since v0.3, so no client of a release loses it.
- The `json-schema` key is `cardamum-card-add`, where v0.3 wrote `cardamum-card-create`: MIGRATION.md says it.
