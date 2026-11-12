---
cairn: change
id: the-sample-is-tested-and-shipped
status: landed
created: 2026-11-12
---

# The sample is tested and shipped

## Why

Fourth change of the-family-standards. The welcome copies the shipped sample in place, so the sample has to parse and has to ship. Cardamum's was a 240-line prose document of five example accounts, never tested and never installed.

## What

- config.sample.toml in Himalaya's shape: one `[accounts.example]` table, the shared options active at their defaults with commented alternatives, each line with its comment; then one commented section per backend or provider: CardDAV, CardDAV without credentials, Google CardDAV, JMAP, Fastmail, Microsoft Graph, Google People, vdir and pimdir. Tokens come from `["ortie", "token", "show", "-a", "example"]`, passwords from `exit 127`.
- `variants` uncomments a section over the active lines, so a section sets no key the top keeps active, and an authentication alternative (`auth = "none"`, `bearer`) is its own section rather than a commented line inside one.
- `the_sample_configuration_parses_with_each_variant` (src/config.rs) parses every variant through `Config::load` and takes `example` as the default account declaring a backend; it failed on a misspelled commented key (`carddav.alpnn`).
- package.nix installs share/doc/cardamum/config.sample.toml; install.sh copies it from a release archive.
- README.md's configuration section reads as Himalaya's: the sample, its sections, Ortie, the shipped path, the welcome, the paths and `-c`.

## Scope / non-goals

The README's provider blocks stay as they are.
