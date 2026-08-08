---
cairn: spec
capability: commands
status: current
---

# Commands

The command tree splits into three groups. The shared API (`addressbook`, `card`) is the cross-protocol least-common-denominator surface, behaving the same whatever backend serves the active account. The protocol-specific APIs each expose the full surface of one backend, including operations the shared API cannot model. The meta commands (`account`, `completions`, `manuals`) cover account inspection, shell completions and man pages.

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

### Requirement: Output streams
All output, data and errors alike, SHALL go to stdout through the printer, distinguished by the exit code; `--json` switches every command to JSON. stderr carries logs and interactive prompts only, so redirecting stdout is always safe.
