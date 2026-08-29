---
cairn: change
id: credentials-resolved-once
status: landed
created: 2026-08-29
---

# One credential command, one spawn per account

## Why

A `password.command` is a command until something runs it, and that used to happen wherever a backend was opened: `build_auth` spawned the CardDAV one, `jmap_http_auth` the JMAP one, and the Graph and People backends their token one. Nothing above them knew a command had run, so nothing could notice two backends naming the same one.

`account check` and the wizard's connection test both reach every backend the account configures. An account naming one `pass` entry from its `carddav` and `jmap` tables therefore paid two `gpg` key unlocks for one entry, and one holding four network backends paid four, for a check whose whole point is to run the same authentication path the other commands take.

## What

pimalaya-config 0.2.0 carries the memo: `Secret::Command` now holds a `command::CommandConfig` (a shell line, or a program and its arguments), which is comparable and cheap to clone where a built `std::process::Command` was neither, and `SecretResolver` keys on it to spawn each distinct command once.

Cardamum builds one resolver where an account is reached as a whole, `AccountCheckCommand::execute` and `test_account`, and threads it through the four network backends. Each of them keeps a plain constructor that resolves on its own, for the protocol-specific commands that open one backend and have nothing to share a credential with.

No configuration changes: both TOML shapes parse and serialize as before, and this moves where a configured command runs rather than what may be configured.
