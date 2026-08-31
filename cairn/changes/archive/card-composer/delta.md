---
cairn: change
id: card-composer
status: landed
created: 2026-08-31
---

# Delta

## ADDED Requirements

### Requirement: A card is edited through a command, never through a pipe
A composer SHALL be a command the configuration names, spawned on the path of a temporary file holding the vCard, with stdin, stdout and stderr all inherited. Cardamum SHALL capture none of the command's streams: a composer that spawns an editor would otherwise hand it a pipe instead of the terminal, and the editor hangs or writes where nothing reads. The command edits the file in place, and cardamum reads it back once the command exits.

The path SHALL be appended as the command's last argument. For a shell line that means interpolating it into the line, single-quoted, since a shell invoked as `sh -c <line> <path>` binds the path to `$0` rather than passing it on.

### Requirement: The composer hands back to a menu, not to a write
When the composer exits, the reader SHALL be asked what to do with what it wrote: `Save`, `Preview`, `Edit again` or `Abort`. Previewing prints the vCard and asks again, so the decision is taken in one place however many times it is deferred.

The entries SHALL be spelled the same on a create and on an update, `Save`, `Preview`, `Edit again` and `Abort`. A non-zero exit status, and a file that came back empty, SHALL abandon the edit without asking.

Saving an unchanged card SHALL be allowed, choosing `Save` being an explicit decision.

### Requirement: A composed card is checked before it is offered
What the composer wrote SHALL be checked against its own version's RFC contract, through vcard-rs's validator. A card that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes, rather than reach the menu. A card built from flags with no source SHALL be checked the same way and refused; a vCard given on the command line SHALL NOT be.

### Requirement: An edit is never lost
When a composed card cannot be written, whether the menu could not be shown or the backend rejected it, the temporary file SHALL be kept and the error SHALL name it, `Cannot edit vCard <path>`. Aborting SHALL keep it too, and name it, unless the composer handed back the seed untouched.

### Requirement: A composer is not spawned under --json
`--json` SHALL refuse to run a composer rather than spawn one. The child inherits cardamum's own stdout, where it would interleave with the JSON payload, and a consumer parsing that output has no terminal to edit in.

### Requirement: A source, the field flags and the composer stack
`card create` and `card update` SHALL take a vCard source, the field flags and `-i/--interactive` together, applied in that order: the source is the card to start from, each flag sets the property it names on it, and `-i` opens the result in the composer. Every combination therefore falls out of one pipeline, and there is no separate compose verb to keep in agreement with them.

The composer SHALL be opt-in through `-i`, never opt-out. A command with no `-i` spawns nothing, which is what keeps both verbs scriptable, and `-i` SHALL bail when no composer is configured rather than fall back to one. `--composer <COMMAND>` SHALL override the configured one for a single invocation, and SHALL require `-i`.

A create given no source SHALL mint the card it starts from rather than open an empty file, so that no composer is asked to invent an identity: it carries a `UID` and a `VERSION` and no more. An update given no source SHALL start from the card the backend holds.

A command given no source, no field flag and no `-i` has nothing to write and SHALL say so, rather than send an unchanged card back to the server.

A card built from flags SHALL be built unchecked. A draft on its way to an editor is not yet a card, and refusing to open one because `FN` is missing is the opposite of what a composer is for.

### Requirement: A minted card carries an identity
A card minted from nothing SHALL carry a `UID`, minted as a fresh `urn:uuid` when `--uid` names none. The pimdir link id derives from it, so a card created without one is unaddressable in the store it lands in, and a composer handed an empty file mints none: tCard seeds an identity for a blank template but preserves what a file already holds, and a graphical editor may do neither.

### Requirement: The field flags are a convenience, not the surface
The field flags SHALL cover the fields `card list` renders and the few every address book carries, and SHALL NOT grow to match what a composer models. `ADR` is out: seven components is where a flag stops being ergonomic. The documentation SHALL say the composer is the complete surface, so the gap reads as the boundary it is rather than as a missing feature.

### Requirement: A field flag sets its property and touches nothing else
Writing a field flag SHALL drop every instance of the property it names and write the flag's own, so a flag sets that property rather than adding to it, and a repeated flag writes one instance per value. Every other line SHALL keep the card's own bytes, the properties no flag covers included.

`N` is the exception, being one property holding several names: `--given-name` and `--family-name` SHALL merge into the card's existing `N` rather than replace it, so setting one does not silently clear the other. All five components of `N` SHALL have a flag, and each SHALL repeat, a component being a comma-separated list (RFC 6350 section 6.2.2).

The name flags SHALL be named for the role, given and family, rather than for the position, first and last. Which of the two is written first is exactly what varies between cultures, which is why RFC 6350, RFC 9553 and CLDR all name the role.

### Requirement: An update guards on the version it read
`card update` given no vCard source SHALL send the ETag of the card it read as `If-Match`, an explicit `--if-match` still winning. Having read the card to change it, the command knows the version it diverged from, and an edit that took a minute SHALL NOT silently overwrite a write that landed during it. A source given on the command line is a rewrite that was asked for, and carries no such guard of its own.

### Requirement: The composer is a command, not a library
`card.composer` SHALL name a command, at the top level and per account, rather than cardamum linking one editor in. What crosses the boundary is a path to a file holding vCard bytes, so the value can be tCard today and a graphical editor tomorrow, and cardamum learns no projection, no TOML and no editor.

It SHALL be a pimalaya-config `CommandConfig`, taking the shell line and the argv list every other command field takes. The command must block until the edit is done, which the documentation SHALL say, since a graphical editor returning immediately is the way this fails in practice.

### Requirement: The connection is opened by the call that needs it
`AddressbookClient` SHALL select its backend from the configuration without connecting, and open the connection on the first call that needs one. A server closes an idle connection and an editor session lasts minutes, so a write landing after one reads the end of a socket whose far side is gone. A command that must read before the editor SHALL drop that connection before spawning the composer.

## MODIFIED Requirements

## REMOVED Requirements
