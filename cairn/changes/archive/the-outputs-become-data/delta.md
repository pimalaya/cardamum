---
cairn: change
change: the-outputs-become-data
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A data command answers a named output type
A command returning data SHALL hand the printer a dedicated type named `<Domain><Target><Verb>Output`, deriving `Display`, `Serialize` and `JsonSchema`, with every public field documented. A write with no payload of its own SHALL print a receipt rather than a sentence: a shared write `{id, queued}` (`WriteReceipt`), the id of the item acted on and whether a sync engine applies the write later, a write in a protocol namespace `{id}` (`IdReceipt`) or `{ids}` (`IdsReceipt`), the ids as they are after the write (a renamed collection's new one), and a `card build -o` `{path, size}`. The sentence is the text form only; data a confirmation used to word (an ETag, member counts) is a key of its own.

Every runnable command SHALL be described in the `json-schema` registry, a command printing one of two types through both (`OneOf`), which a test checks over the clap command tree.

A command whose two shapes are one invocation SHALL answer one type covering both, untagged so each shape serializes exactly as it would alone.

## REMOVED Requirements
