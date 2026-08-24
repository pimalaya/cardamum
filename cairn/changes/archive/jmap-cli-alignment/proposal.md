---
cairn: change
id: jmap-cli-alignment
status: landed
created: 2026-08-09
---

# Align the JMAP CLI with Himalaya, and stop leaking internals through it

The 2026-08-09 Fastmail JMAP run ([jmap-fastmail.md](../../../spec/testing/jmap-fastmail.md), [jmap-specific.md](../../../spec/testing/jmap-specific.md)) found one command that cannot run at all and a handful of places where an internal shape reaches the user.

**S4: `jmap contact-card copy` aborts before doing anything.** It declares `--from-account` with `short = 'a'`, which collides with the global `-a/--account`. In a debug build clap asserts and the process dies with exit 101; in a release build, where those assertions are compiled out, the duplicate short is simply ambiguous. The command has therefore never run. Himalaya's twin (`jmap email copy`) already has the argument model to copy: ids positional and repeatable, everything else long-only.

**S2: `jmap address-book get` takes its ids as a repeatable `--id` flag** while `jmap contact-card get` takes them positionally, and both help texts read "…by id". Himalaya is positional throughout.

**S1: `contact-card create` and `update` print empty fields.** Both render through the same three-field report, but a JMAP `set` response only echoes server-set properties: create returns id, created and prodId, so name and address-books print blank, and update returns an object with nothing in it, so the whole block is blank while the operation succeeded.

**J7: a rejected write prints its `SetError` as Rust `Debug`**, e.g. `Forbidden { description: Some("AddressBooks may not be created") }` or `InvalidProperties { description: None, properties: ["vCardProps"] }`. Himalaya solved this already, with a `JmapSetError` trait and a `format_set_error` renderer.

**J6: Fastmail rejects the standard `vCardProps` member**, so a vCard carrying any property JSContact cannot model fails to write, with nothing but the raw `InvalidProperties` to go on. The server is at fault (RFC 9555 §2.15 defines the member), but the user is left guessing which property in their card is to blame.

## What changes

- `contact-card copy` follows Himalaya: `<CARD-ID>...` positional and repeatable, `--from-account` and `--to-address-book` long-only, the latter repeatable since a JMAP copy can land in several books. The `-a` collision goes with it.
- `address-book get` takes its ids positionally, still optional so that omitting them keeps fetching every book.
- The card report prints only the fields the returned object actually carries, and an update whose response carries nothing at all reports success in words instead of printing an empty block.
- A new jmap/error.rs ports Himalaya's `JmapSetError` / `format_set_error` for the three cardamum error types, so a rejected write reads `invalidProperties (\`vCardProps\`): <server description>`.
- On top of that rendering, a rejected write naming `vCardProps` gains an actionable hint listing the vCard properties that landed there, in the spirit of the CardDAV missing-UID hint. Nothing is dropped silently: the write still fails.

## What does not change

`--json` keeps emitting the raw native payload everywhere, per the raw-faithful requirement. Whether cardamum should stash or drop the properties Fastmail refuses is a separate product decision; this change only makes the failure legible.
