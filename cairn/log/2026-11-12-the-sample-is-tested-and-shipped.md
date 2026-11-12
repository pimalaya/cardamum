---
cairn: log
change: the-sample-is-tested-and-shipped
date: 2026-11-12
---

# The sample is tested and shipped

config.sample.toml is one `[accounts.example]` table and nine commented sections (CardDAV, CardDAV without credentials, Google CardDAV, JMAP, Fastmail, Microsoft Graph, Google People, vdir, pimdir), every variant parsed by `the_sample_configuration_parses_with_each_variant`, which failed on a misspelled commented key before the fix was reverted. package.nix installs it as share/doc/cardamum/config.sample.toml and install.sh copies it; the README configuration section reads as Himalaya's.

Capabilities moved: config (added "One sample configuration, with a section per backend"), packaging (added "The sample ships beside the binary").
