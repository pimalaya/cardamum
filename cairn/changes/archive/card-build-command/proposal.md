---
cairn: change
id: card-build-command
status: landed
created: 2026-08-31
---

# A vCard can be built without a backend to send it to

## Why

The field flags made a card writable from the command line, and left it unseeable. `card create --full-name "Jane Doe" --email work:jane@corp.example` mints the card, checks it and appends it to an addressbook in one go, so the first look anyone gets at what those flags produced is the card the server already holds. `-i` is the only preview, and it costs an editor, a terminal and a configured composer.

That is also a gap in what the flags compose with. Both write verbs take `-` as a source, so a vCard arriving on standard input is already a first-class way in, and nothing in the tree produces one. The obvious pipelines, seeding a card from a template, editing a stored card through a filter, generating fixtures for the provider test plan, all end at the same missing half.

[card-composer](../archive/card-composer/proposal.md) ruled a separate verb out, and it was right about what it was ruling out. A second way to *write* a card, kept in agreement with `create` and `update` forever, buys nothing. A command that writes to nobody is a different thing: it is the pipeline those two share, stopped one step before the client.

## What

**`card build`**, taking the same vCard source and the same field flags as `card create`, printing the resulting vCard on stdout and touching no backend.

**It resolves no account.** `Command::Card` builds an `AddressbookClient` before the subcommand runs, so every card subcommand today needs a configuration file and a resolvable addressbook. Formatting a vCard needs neither, and a build that bails with `No default account found` would be the feature failing at exactly the moment it is most useful, on a machine that has never run the wizard. Card dispatch therefore resolves the account per subcommand rather than ahead of all of them.

**It has no `-i`.** The composer is what `create` and `update` already offer, and wiring it here is what would drag the account back in, `card.composer` living in the configuration. Building a card into a file and opening it afterwards is two commands that already exist.

**It checks what `create` would check.** A card built from flags with no source is validated and refused, exactly as `create` refuses one; a vCard given as a source passes through unchecked, exactly as `create` passes it. Any other rule opens a hole: `card build --full-name X | card create -` arrives at `create` as a source, which `create` does not check, so a `build` that checked less than `create` would be a way of getting an invalid card past the guard by piping it.

## What it is not

`card build` is not a dry run of `card create`. It knows nothing of the addressbook, the backend or the ETag, and a card it printed can still be refused by a server. It answers what the flags produce, which is the question `-i` answers today at the price of an editor.

It is not an output destination either. Sending the card somewhere other than stdout was considered as `-o` on the two write verbs, and rejected: a flag that quietly turns `create` into a command that creates nothing lies about the verb it is attached to, cannot say whether it means *instead of* the server or *in addition to* it, and still drags the account resolution and the addressbook lookup behind it. Redirection covers the file case.

The field flags stay what they were, a convenience over the common properties rather than the whole of vCard. `card build` makes their result visible; it does not make them grow.
