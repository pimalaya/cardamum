---
cairn: log
change: card-build-command
landed: 2026-08-31
---

# A vCard can be built without a backend to send it to

The field flags made a card writable from the command line and left it unseeable: `card create --full-name "Jane Doe"` minted, checked and appended the card in one go, so the first look anyone got at what the flags produced was the card the server already held. `card build` is that pipeline stopped one step early.

**The command is `create` minus the client** (shared/card/build.rs): the same vCard source, the same `CardFieldsArgs`, the same `--vcard-version`, and a `CardBuildOutput` whose `Display` writes the raw vCard so it pipes, `--json` answering `{"contents"}`. Both write verbs already take `-` as a source, so `card build ... | card create -` composes, and `card read <ID> | card build --title CTO -` previews an update without a verb of its own.

**It resolves no account** (shared/card/cli.rs): `Command::Card` built an `AddressbookClient` before the subcommand ran, so every card subcommand needed a configuration file and a resolvable addressbook. Card dispatch now resolves per subcommand, `build` resolving nothing, which is what lets it format a vCard on a machine that has never run the wizard. Verified with an empty `HOME` and `XDG_CONFIG_HOME`: the card prints, exit 0.

**It checks what a create checks** (shared/card/vcard.rs): the refusal moved out of `card create` into `ensure_valid`, and both call it for a card built from flags with no source. A vCard given as a source still passes through untouched on both. Any laxer rule here would have been a hole rather than a convenience: a built card reaches `create` as a source, which `create` does not check, so `build | create -` would have been the way around the guard `create --full-name` enforces.

**No composer** (`-i` is absent by design): it is what the two write verbs already carry, and wiring it here would have dragged the configuration back in, `card.composer` living there. Building into a file and opening it afterwards is two commands that already exist.

The composer pair itself was hoisted while passing: `-i/--interactive` and `--composer` were spelled twice, once in create.rs and once in update.rs, and are now `CardComposerArgs` in shared/arg.rs, flattened into both.

Capabilities moved: commands (one requirement added, two modified).

Not done here: the feature matrix does not build on this tree, `--no-default-features` and each backend alone failing on io-pim-discovery 0.7.0 imports and on client `connect` signatures. Those failures predate this change and touch none of the files it moved; `--all-features` checks, clippies and tests clean.
