---
cairn: delta
change: a-card-is-added
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: A card is added
(commands) The unified verb creating a card SHALL be `card add`, as Himalaya's `message add`, its MCP tool `card_add` and its schema `cardamum-card-add`. `card create`, the v0.3 name, SHALL stay accepted as a hidden alias, and `card new` as a visible one. The capability it reaches is `card.add`, pimdir's `contacts.card.add`.

## MODIFIED Requirements

### Requirement: Tools are the classified unified API
(mcp) `card_add` replaces `card_create` in the list of the tools `--allow-write` adds. Every leaf of `addressbook` and `card` SHALL be either a tool or excluded by name (`card build`, a local formatter writing files), and the instructions SHALL name no tool the server does not define, each checked by a test.

### Requirement: A queued write says it is queued
(commands) `card add` replaces `card create` in the requirement and its scenario, as in every other requirement naming the verb (commands: "A data command answers a named output type", "A composed card is checked before it is written", "A source, the field flags and the composer stack", "A card can be built without an account", "The JSON contract is described and versioned"; backends: "Card ids are backend-native and verbatim", "The connection is opened by the call that needs it").

## REMOVED Requirements

None.
