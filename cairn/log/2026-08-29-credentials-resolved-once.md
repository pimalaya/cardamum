---
cairn: log
change: credentials-resolved-once
landed: 2026-08-29
---

# One credential command, one spawn per account

A `password.command` is a command until something runs it, and that used to happen wherever a backend was opened. `build_auth` spawned the CardDAV credential, `jmap_http_auth` the JMAP one, and the Graph and People backends their token one; none of them knew another had just run the same command. `account check` and the wizard's connection test reach every backend the account configures, so an account naming one `pass` entry from its `carddav` and `jmap` tables paid two `gpg` key unlocks for one entry, and a four-backend account paid four.

**Memo** (pimalaya-config 0.2.0): `Secret::Command` now carries a `command::CommandConfig`, a shell line or a program with its arguments, instead of a built `std::process::Command`. The configured shape is comparable, hashable and cheap to clone, which the built command is not, so `SecretResolver` can key on it and spawn each distinct command once. The two shapes never compare across: a shell line and its argv spelling are two commands whatever they end up running. Both TOML shapes parse and serialize exactly as before, so no configuration moved.

**Resolution paths**: `open_carddav_client_with`, `JmapBackend::new_with`, `MsgraphBackend::new_with` and `PeopleBackend::new_with` take the resolver and pass it down to `build_auth` and `jmap_http_auth`, where a `secret.get()` became a `resolver.resolve(secret)`. Each keeps its plain constructor, which resolves through a resolver of its own, for the protocol-specific commands and the shared client, all of which open one backend and have nothing to share a credential with.

**Callers**: `AccountCheckCommand::execute` and `test_account` build one `SecretResolver` for the account they reach and thread it through every backend, so the check that walks the same authentication path as the other commands stops paying for the walk twice. The resolver is a local of both, dropped with the account: it holds plaintext, and nothing keeps it on a client.

**Wizard** (`wizard/secret.rs`): `command_secret` and `shell_secret` build a `CommandConfig::Argv` and a `CommandConfig::Shell` where they built a `Command` and called `command::shell`. The empty argv and the blank shell line are still refused, and the generated TOML is unchanged: an array for a known keyring provider or token broker, a string for a hand-typed command.

Verified: 56 unit tests green with every feature, and each backend builds and resolves on its own. Also fixed a test-only gate this uncovered, `AccountConfig` being imported under `vdir` alone while the CardDAV rendering test needs it, so `cargo test --features carddav` compiles.

Spec updated: `config` (ADDED: "A credential command is spawned once per account").
