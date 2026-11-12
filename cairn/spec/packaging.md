---
cairn: spec
capability: packaging
status: current
---

# Packaging

Cardamum is an application, the top layer of the Pimalaya stack: it drives the sans-I/O io-* libraries through their blocking `*Std` clients and renders their results, the CLI plumbing, the TOML configuration and the blocking stream runtime coming from pimalaya-cli, pimalaya-config and pimalaya-stream.

### Requirement: Released dependencies, patched only while unreleased
Dependencies SHALL name released versions. A `[patch.crates-io]` entry is allowed only while a needed change rides unreleased upstream, the cairn change adding it SHALL say what it waits on (Cargo.toml carrying no comment in its dependency or patch sections), and it SHALL be dropped as that crate publishes.

### Requirement: The sample ships beside the binary
Packages and release archives SHALL install config.sample.toml as share/doc/cardamum/config.sample.toml, where the welcome finds it, and install.sh SHALL copy it under its prefix.

### Requirement: Dependencies name their features
Every dependency, build and dev ones included, SHALL take `default-features = false` and name the features its items need, and the manifest SHALL list no dependency the code does not name.

### Requirement: Dependency checks pass clean
`cargo deny check` SHALL pass its advisories, bans, licences and sources checks over the release targets deny.toml lists, the licences allowed being the ones the tree carries.

### Requirement: Hosts are ASCII
A server or proxy URL with a special scheme (`http`, `https`) SHALL be refused when its host is not ASCII, its Punycode form working: IDNA goes through `idna_adapter` 1.0.0, without the ICU4X crates.

### Requirement: Feature-gated backends
Every backend SHALL sit behind its own cargo feature (`carddav`, `jmap`, `msgraph`, `gpeople`, `vdir`, `pimdir`), so a build ships only the protocols it needs. A protocol command and its backend adapter compile only when its feature is on; the unified command tree compiles in every build.

### Requirement: Every backend builds alone
clippy `--all-targets -D warnings` SHALL pass on the default build, all features, `rustls-ring` with no backend, `native-tls,jmap`, `rustls-ring,vdir,pimdir` and each backend alone with `rustls-ring`, checked in CI.

### Requirement: Live provider suites
tests/google.rs, tests/microsoft.rs and tests/fastmail.rs SHALL run the built binary against Google (People, CardDAV), Microsoft Graph and Fastmail (JMAP, CardDAV), ignored unless asked: `anonymous::` with a made-up token on every push, `credentialed::` read-only on the test accounts with tokens the suite mints, in the cardamum repository once its secrets exist. A token SHALL reach the binary through its environment only, and no assertion SHALL print a card's value.
