---
cairn: log
change: the-wizard-leaves
date: 2026-11-12
---

# The wizard leaves

src/wizard/ (2,421 lines), `configure` and its `wizard` alias, the `wizard` feature, the first-run offer, the `cardamum-configure` schema, `test_account` and the config renderer (`render`, its key orders, `is_default`, every `Serialize` derive) are gone. A bare `cardamum` prints the help; a command finding no configuration fails naming the path and the sample. io-pim-discovery stays behind `carddav` with `pacc`, `rfc6764` and `stream` for `carddav.discover`; shellexpand left. 91 tests pass (the wizard's 24 went with it), clippy `-D warnings` passes on all features and on carddav, jmap and vdir alone.

Capabilities moved: wizard (removed whole), config ("Account resolution is explicit or default", "A credential command is spawned once per account", "A DAV server may ask for no credentials"), commands ("The top-level help orients a newcomer", "Subcommands carry their own data and chain").
