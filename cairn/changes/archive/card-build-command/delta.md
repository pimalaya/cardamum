---
cairn: change
id: card-build-command
status: landed
created: 2026-08-31
---

# Delta

## ADDED Requirements

### Requirement: A card can be built without an account
`card build` SHALL take the same vCard source and the same field flags as `card create`, apply them in the same order, and print the resulting vCard on stdout. It SHALL reach no backend.

It SHALL resolve no account and SHALL read no configuration, so that a vCard can be formatted on a machine that has none. The account a card subcommand runs against SHALL therefore be resolved by that subcommand rather than ahead of the whole `card` family.

It SHALL check what `card create` would check: a card built from flags with no source is validated and refused, a vCard given as a source passes through untouched. A weaker rule would make `card build ... | card create -` a way past the guard, the piped card arriving at `create` as a source, which `create` does not check.

It SHALL NOT offer the composer. `-i` is what the two write verbs already carry, and the composer command lives in the configuration this one deliberately does not read.

A build given no source and no field flag has nothing to build and SHALL say so.

## MODIFIED Requirements

### Requirement: A composed card is checked before it is offered
What the composer wrote SHALL be checked against its own version.s RFC contract before the menu is shown, through vcard-rs`s validator rather than a look at its first line. Reading a card is liberal and this is the strict half: a 4.0 card missing its required `FN`, a 2.1 one missing its required `N`, a property the version does not define, are all caught here rather than by the server or by nobody.

A card that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes, rather than reach the menu at all. `Save` is therefore offered only for a card that would be accepted, and a broken one leads straight back to where it can be fixed.

A card built from flags with no source SHALL be checked the same way and refused, naming the violations, whether it was `card create` or `card build` that built it. A vCard given on the command line SHALL NOT be, going to the backend as it was written: that is the promise the protocol-specific commands already make, and second-guessing a card someone handed over is not cardamum.s to make.

### Requirement: A source, the field flags and the composer stack
`card create` and `card update` SHALL take a vCard source, the field flags and `-i/--interactive` together, applied in that order: the source is the card to start from, each flag sets the property it names on it, and `-i` opens the result in the composer. Every combination therefore falls out of one pipeline, which `card build` stops one step early rather than restating: no verb beside these two writes a card.

The composer SHALL be opt-in through `-i`, never opt-out. A command with no `-i` spawns nothing, which is what keeps both verbs scriptable, and `-i` SHALL bail when no composer is configured rather than fall back to one. `--composer <COMMAND>` SHALL override the configured one for a single invocation, and SHALL require `-i`. The pair SHALL be one shared argument, spelled once for both verbs.

A create given no source SHALL mint the card it starts from rather than open an empty file, so that no composer is asked to invent an identity: it carries a `UID` and a `VERSION` and no more. An update given no source SHALL start from the card the backend holds.

A command given no source, no field flag and no `-i` has nothing to write and SHALL say so, rather than send an unchanged card back to the server.

A card built from flags SHALL be built unchecked. A draft on its way to an editor is not yet a card, and refusing to open one because `FN` is missing is the opposite of what a composer is for.

## REMOVED Requirements

None.
