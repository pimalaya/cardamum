---
cairn: spec
capability: packaging
status: current
---

# Packaging

Cardamum is an application, the top layer of the Pimalaya stack: it drives the sans-I/O io-* libraries through their blocking `*Std` clients and renders their results, the CLI plumbing, the TOML configuration and the blocking stream runtime coming from pimalaya-cli, pimalaya-config and pimalaya-stream.

### Requirement: Released dependencies, patched only while unreleased
Dependencies SHALL name released versions. A `[patch.crates-io]` entry is allowed only while a needed change rides unreleased upstream, the cairn change adding it SHALL say what it waits on (Cargo.toml carrying no comment in its dependency or patch sections), and it SHALL be dropped as that crate publishes.
