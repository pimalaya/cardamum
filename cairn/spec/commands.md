---
cairn: spec
capability: commands
status: current
---

# Commands

The command tree splits into three groups. The shared API (`addressbook`, `card`) is the cross-protocol least-common-denominator surface, behaving the same whatever backend serves the active account. The protocol-specific APIs each expose the full surface of one backend, including operations the shared API cannot model. The meta commands (`configure`, `account`, `completion`, `manual`, `json-schema`) cover account generation and inspection, shell completions, man pages and the JSON Schemas of the structured output.

### Requirement: The top-level help orients a newcomer
The top-level help SHALL frame the first run in its long description, naming the bare invocation and `cardamum configure` as the two ways to generate an account, and SHALL end with the shared Pimalaya footer (`pimalaya_cli::footer`) pointing at the issue tracker and the sponsoring page, as every other Pimalaya CLI does.

### Requirement: Subcommands carry their own data and chain
Each subcommand SHALL be a clap-derived struct holding its own arguments, with an `execute(self, printer, client)` method. `Command::execute` in [cli.rs](../../src/cli.rs) is the single dispatch point: it resolves the account (proposing the wizard when no config exists), builds the appropriate client, and hands it to the subcommand.

### Requirement: The doc comment is the help page
Each command's doc comment SHALL be its help text: the first paragraph (two lines maximum) is the short summary `-h` shows, and the full text `--help` shows SHALL end with the command's JSON output shape. `cardamum <command> --help` is therefore the canonical usage reference for both humans and agents, and the README documents no per-command usage.

### Requirement: Shared commands stay least-common-denominator
The shared `addressbook` and `card` commands SHALL expose only what every backend can model, and their documentation SHALL stay protocol-agnostic. A partial-coverage concept (CardDAV ETags, JMAP m:n memberships, Graph delta) belongs to a protocol-specific command instead of being ejected from the shared API or emulated everywhere.

### Requirement: Specific commands mirror their protocol
A protocol-specific command set SHALL match the remote protocol's own structure and vocabulary rather than the shared API's, following Himalaya's direction. The shape follows the protocol: CardDAV (WebDAV) is a flat list of WebDAV method names, JMAP, Microsoft Graph and Google People are nested by resource with sub-verbs naming the protocol methods, and vdir is flat collection verbs plus an `item` subcommand. Verb names track the protocol, so JMAP uses `destroy` while Graph and People use `delete`.

pimdir has no protocol-specific surface by design: it is a store rather than a protocol, and its operator commands ship as the separate `pimdir` binary in io-pimdir.

### Requirement: Specific commands are raw-faithful
A specific command SHALL surface the native ids and metadata the shared layer normalizes away (ETags, CTags, sync tokens, hrefs, the Graph `changeKey`, the People `etag` and `resourceName`), and SHALL make the incremental-sync primitive its headline (`report sync`, `changes`, `delta`). Default human output is a friendly table; `--json` SHALL emit the raw native payload, and create and update SHALL accept the raw native representation. Each backend keeps a raw escape hatch: a raw XML body for CardDAV, raw JSON in and out plus a generic request passthrough for the object backends. Precondition control is offered only where the protocol has it (CardDAV `--if-match` and `--if-none-match`).

### Requirement: Specific commands take their ids positionally
A protocol-specific command SHALL take the ids it operates on as positional arguments, repeatable where the protocol accepts several, and reserve flags for the rest, mirroring Himalaya. A short option SHALL NOT be given to an argument whose letter the global options already use: clap asserts on the collision in a debug build and silently makes the letter ambiguous in a release one, so the command never runs either way.

### Requirement: A command prints what the server returned
A command SHALL render only the fields the server's response actually carries, and SHALL report success in words when the response carries nothing to show, rather than printing empty field slots. This matters most on the write path, where a JMAP `set` response echoes only the properties the server itself decided, and where a REST delete answers 204 with no body at all: text output SHALL then print nothing. `--json` still emits the raw native payload.

### Requirement: An incremental-sync command can resume
A protocol-specific command exposing its backend's incremental sync SHALL accept the resume token that sync hands back, in whatever shape the protocol uses (a Graph `@odata.deltaLink` URL, a People sync token, a JMAP or DAV state string). A command that documents a resume SHALL provide the flag for it.

### Requirement: An empty addressbook id is rejected
The shared `-k/--addressbook` resolver SHALL reject an empty id rather than pass it to a backend, where it silently addresses the collection instead of a member.

### Requirement: A partial write says so
When a backend cannot carry out part of a write, and the protocol offers no way to insist, the command SHALL report what was left undone rather than print a plain success. On Google People an unmappable vCard property lives in a `clientData` entry the API will not clear, so an update dropping one keeps it, and the command names it.

### Requirement: A displayed count is the server's
A table SHALL NOT render an absent value as a default that reads as fact. A count the API returns only on request SHALL be requested, or left blank.

### Requirement: A command resolves its addressbook before acting
A shared command SHALL resolve the addressbook it was given before reading or writing through it, and SHALL fail naming it when it does not exist. Listing an unknown addressbook SHALL NOT read as empty, and writing into one SHALL NOT create it.

### Requirement: A rejected write is legible
A rejected protocol write SHALL surface as prose: the server's own error type and description, plus the properties it named, never a Rust `Debug` rendering of the error value. Where the rejection has a known cause the user can act on, the message SHALL name it, as the CardDAV missing-UID hint and the JMAP `vCardProps` hint do.

### Requirement: Output streams
All output, data and errors alike, SHALL go to stdout through the printer, distinguished by the exit code; `--json` switches every command to JSON. stderr carries logs and interactive prompts only, so redirecting stdout is always safe.

### Requirement: A data command answers a named output type
A command returning data SHALL hand the printer a dedicated type named `<Domain><Target><Verb>Output`, deriving `Display`, `Serialize` and `JsonSchema`, with every public field documented. `pimalaya_cli::printer::Message` SHALL carry confirmations only: it serializes as one prose string, so a data command using it yields a `--json` payload no consumer can read.

A command whose two shapes are one invocation SHALL answer one type covering both, untagged so each shape serializes exactly as it would alone.

### Requirement: JSON output keys are camelCase
Every output type SHALL carry `serde(rename_all = "camelCase")`, so a `--json` key spelling more than one word reads `addressbookId` rather than `addressbook_id`. That is the convention of the wire formats the backends speak, and unlike a hyphenated key it needs no quoting in a `jq` path. The configuration vocabulary is a separate one and stays kebab-case.

A field renamed onto a provider's own wire key SHALL keep that spelling, a container attribute never reaching a field carrying its own `rename`: `@odata.nextLink`, `nextPageToken`, `contactGroups` and the JMAP `list` are the provider quoted back to the caller, not ours to restyle.

### Requirement: The JSON output of every data command has a schema
Every output type SHALL be registered in [json_schema.rs](../../src/json_schema.rs) under its CLI invocation path, hyphen-joined and prefixed `cardamum-`, and the `json-schema` command (aliased `json-schemas`) SHALL print one or write one file per entry. A registered key naming no command is a test failure, so the registry cannot drift from the tree.

A wire object the protocol crate exposes without a `JsonSchema` of its own SHALL be described as raw JSON rather than left out: the payload is the provider's to define, and claiming a shape we do not own would be worse than claiming none.

### Requirement: comfy-table is reached through the toolkit
Table rendering SHALL go through `pimalaya_cli::table` rather than a direct `comfy-table` dependency, so the toolkit owns the version every Pimalaya CLI draws with.

### Requirement: A card is edited through a command, never through a pipe
A composer SHALL be a command the configuration names, spawned on the path of a temporary file holding the vCard, with stdin, stdout and stderr all inherited. Cardamum SHALL capture none of the command's streams: a composer that spawns an editor would otherwise hand it a pipe instead of the terminal, and the editor hangs or writes where nothing reads. The command edits the file in place, and cardamum reads it back once the command exits.

The path SHALL be appended as the command's last argument. For a shell line that means interpolating it into the line, single-quoted, since a shell invoked as `sh -c <line> <path>` binds the path to `$0` rather than passing it on.

### Requirement: The composer hands back to a menu, not to a write
When the composer exits, the reader SHALL be asked what to do with what it wrote: `Save`, `Preview`, `Edit again` or `Abort`. Previewing prints the vCard and asks again, so the decision is taken in one place however many times it is deferred, and every path out of an interactive edit is one of the four rather than an inference from what the bytes look like.

The entries SHALL be spelled the same on a create and on an update. At that point the card is written or it is not, and which verb asked is already on the command line. `Abort` SHALL be preferred to `Cancel`, which reads as though it dismissed the menu rather than the edit behind it.

A non-zero exit status, and a file that came back empty, SHALL abandon the edit without asking: both are a person already having said no.

Saving an unchanged card SHALL be allowed. Choosing `Save` is an explicit decision, and second-guessing it because the bytes did not move would refuse what was just asked for.

### Requirement: A composed card is checked before it is offered
What the composer wrote SHALL be checked against its own version.s RFC contract before the menu is shown, through vcard-rs`s validator rather than a look at its first line. Reading a card is liberal and this is the strict half: a 4.0 card missing its required `FN`, a 2.1 one missing its required `N`, a property the version does not define, are all caught here rather than by the server or by nobody.

A card that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes, rather than reach the menu at all. `Save` is therefore offered only for a card that would be accepted, and a broken one leads straight back to where it can be fixed.

A card built from flags with no source SHALL be checked the same way and refused, naming the violations, whether it was `card create` or `card build` that built it. A vCard given on the command line SHALL NOT be, going to the backend as it was written: that is the promise the protocol-specific commands already make, and second-guessing a card someone handed over is not cardamum.s to make.

### Requirement: An edit is never lost
When a composed card cannot be written, whether the menu could not be shown or the backend rejected it, the temporary file SHALL be kept and the error SHALL name it: `Cannot edit vCard <path>`, which says what failed and where the work is in one line, the path being the recovery. A person who spent a minute in an editor never loses it to a failure that happened afterwards.

Aborting SHALL keep the file too, and name it, unless the composer handed back the seed untouched: a card nobody worked on is nothing to lose, and removing it is what keeps the temporary directory clear of every abandoned run.

### Requirement: A composer is not spawned under --json
`--json` SHALL refuse to run a composer rather than spawn one. The child inherits cardamum's own stdout, where it would interleave with the JSON payload, and a consumer parsing that output has no terminal to edit in.

### Requirement: A source, the field flags and the composer stack
`card create` and `card update` SHALL take a vCard source, the field flags and `-i/--interactive` together, applied in that order: the source is the card to start from, each flag sets the property it names on it, and `-i` opens the result in the composer. Every combination therefore falls out of one pipeline, which `card build` stops one step early rather than restating: no verb beside these two writes a card.

The composer SHALL be opt-in through `-i`, never opt-out. A command with no `-i` spawns nothing, which is what keeps both verbs scriptable, and `-i` SHALL bail when no composer is configured rather than fall back to one. `--composer <COMMAND>` SHALL override the configured one for a single invocation, and SHALL require `-i`. The pair SHALL be one shared argument, spelled once for both verbs.

A create given no source SHALL mint the card it starts from rather than open an empty file, so that no composer is asked to invent an identity: it carries a `UID` and a `VERSION` and no more. An update given no source SHALL start from the card the backend holds.

A command given no source, no field flag and no `-i` has nothing to write and SHALL say so, rather than send an unchanged card back to the server.

A card built from flags SHALL be built unchecked. A draft on its way to an editor is not yet a card, and refusing to open one because `FN` is missing is the opposite of what a composer is for.

### Requirement: A card can be built without an account
`card build` SHALL take the same vCard source and the same field flags as `card create`, apply them in the same order, and print the resulting vCard on stdout. It SHALL reach no backend.

It SHALL resolve no account and SHALL read no configuration, so that a vCard can be formatted on a machine that has none. The account a card subcommand runs against SHALL therefore be resolved by that subcommand rather than ahead of the whole `card` family.

It SHALL check what `card create` would check: a card built from flags with no source is validated and refused, a vCard given as a source passes through untouched. A weaker rule would make `card build ... | card create -` a way past the guard, the piped card arriving at `create` as a source, which `create` does not check.

It SHALL NOT offer the composer. `-i` is what the two write verbs already carry, and the composer command lives in the configuration this one deliberately does not read.

A build given no source and no field flag has nothing to build and SHALL say so.

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
