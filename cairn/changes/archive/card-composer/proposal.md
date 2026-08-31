---
cairn: change
id: card-composer
status: landed
created: 2026-08-31
---

# A card is written from flags and refined in a command of your choosing

## Why

Cardamum could read a vCard and write a vCard, and had no way to make one. `card create` took a path, `-`, or literal `BEGIN:VCARD` bytes, all of which assume the card already exists somewhere. Writing one by hand means knowing that `N` is family-first, that `ADR` has seven components and that a comma inside a value is escaped, which is the opposite of what a contacts CLI is for.

Himalaya shipped an editor path and reverted it in [662bd26](https://github.com/pimalaya/himalaya/commit/662bd26). Its composer named a shell command, spawned it through `sh -c` and captured its stdout as the draft. A composer that spawns `$EDITOR` then inherits a pipe instead of the terminal, so the editor either hangs or writes into a stream nobody reads. That commit message named the cure and did not take it: an intermediate file.

The cure is the whole of this change. Cardamum writes the vCard to a temporary file, spawns the configured command on it with every stream inherited, and reads the file back. Nothing is captured, so nothing can deadlock, and the command is free to be a terminal editor, a wrapper script or a GUI.

Linking a projection library instead was considered and rejected: it makes one editor the editor. The value of a command is that today it is tCard and tomorrow it is whatever the person prefers.

## What

**`card.composer`**, at the global and account levels, holding a pimalaya-config `CommandConfig`: a shell line or an argv list. Cardamum appends the path of the temporary vCard as the command's last argument. `composer = "tcard edit"` therefore runs `tcard edit /tmp/cardamum-<uuid>.vcf`, which tCard already edits in place.

**One pipeline on the two write verbs**, rather than a third verb beside them. `card create` and `card update` each take a vCard source, the field flags and `-i/--interactive`, applied in that order: the source is the card to start from, each flag sets the property it names on it, and `-i` opens the result in the composer. Every combination falls out of that, and there is no compose verb to keep in agreement with the other two.

The field flags are `--full-name`, `--given-name`, `--family-name`, `--middle-name`, `--name-prefix`, `--name-suffix`, `--nickname`, `--organization`, `--title`, `--birthday`, `--email work:a@b`, `--phone cell:+1-555-0100`, `--url`, `--note` and `--uid`, which is for a card what Himalaya's `message compose` flags are for a message.

**The composer is opt-in through `-i`, never opt-out.** A command with no `-i` spawns nothing, so both verbs stay scriptable without a flag to turn the composer off, and `-i` bails when none is configured instead of quietly falling back to one.

**A create with no source mints its card**, carrying a `UID` and a `VERSION` and no more, rather than opening an empty file. tCard seeds an identity for a blank template but preserves what a file already holds, so a composer handed an empty file returns a card with neither, and the pimdir link id derives from that `UID`.

**An update with no source starts from the stored card**, and sends the ETag it read as `If-Match`. `update` had no base to guard against, so an edit taking a minute could silently overwrite a write that landed in that minute. `--if-match` still wins, and a source given on the command line is a rewrite that was asked for and carries no guard of its own.

## What a flag means on a card that already exists

A flag sets the property it names: every instance the card carried is dropped and the flag's own is written, a repeated flag writing one instance per value. Every other line keeps the card's own bytes.

That rule is worth stating plainly because it is a real edge: `--email` on a card holding three addresses leaves one. Adding to a property rather than setting it is what the composer is for, and a second flag spelling for it would be two ways to do the same thing.

`N` is the exception, being one property holding several names. `--given-name` and `--family-name` merge into the existing `N` rather than replace it, because silently clearing someone's given name while setting their family name is not an edge, it is a bug.

## Who writes the bytes

vcard-rs, promoted from an optional dependency to an unconditional one. Cardamum needs a real vCard writer now, and hand-rolling one gets RFC 6350 section 3.4 escaping or 75-octet folding wrong in the way that silently corrupts a name holding a comma. It is already in the tree for the JMAP, Graph and People backends.

A card built from flags is built unchecked. A draft on its way to a composer is not yet a card, and refusing to open the editor because `FN` is missing would defeat the point. Validation belongs to the composer and to the server.

## What this is not

The composer is not a renderer. `card read` still prints the raw vCard, and a `card.reader` reusing this same mechanism is a separate change.

The flag vocabulary is not the whole of vCard and never will be. It covers the fields `card list` renders plus the few every address book has, and `ADR` is deliberately absent: seven components is where a flag stops being ergonomic. The composer is the complete surface, and the specification says so, so the gap is not a bug.

Cardamum learns no TOML, no projection and no editor. What crosses the boundary is a path to a file holding vCard bytes.
