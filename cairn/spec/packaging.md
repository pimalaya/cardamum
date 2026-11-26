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
A server or proxy URL with a special scheme (`http`, `https`) SHALL be refused when its host is not ASCII, its Punycode form working: IDNA goes through `idna_adapter` 1.0.0, without the ICU4X crates. A CardDAV `server` or `home` and a JMAP `server` SHALL be refused naming the host and the way out, "Host `bücher.de` is not ASCII: write it in its Punycode form (xn--...)": `home` when the configuration loads, `server` when the account connects. A CardDAV `server` failing to parse that way SHALL never be prefixed with `https://`, which would read `https` as its host.

#### Scenario: An internationalised CardDAV server
- GIVEN an account holding `carddav.server = "https://bücher.example/dav/"`
- WHEN a command reaches CardDAV
- THEN it fails naming `bücher.example` as not ASCII and its Punycode form as the way out, sending nothing

### Requirement: Feature-gated backends
Every backend SHALL sit behind its own cargo feature (`carddav`, `jmap`, `msgraph`, `gpeople`, `vdir`, `pimdir`), so a build ships only the protocols it needs. A protocol command and its backend adapter compile only when its feature is on; the unified command tree compiles in every build.

### Requirement: Only the pimdir backend links SQLite
The `pimdir` feature SHALL take io-pimdir's `client`, `search` and `contact`; `carddav`, `jmap`, `msgraph`, `gpeople` and `vdir` SHALL take its `contact` alone (the card matcher, no SQLite, no mail or calendar parser). The tests take its `indexer` as a dev-dependency, so no release binary carries the index refresh, and take rusqlite to mark a store or an index with another format version.

#### Scenario: A CardDAV-only build
- GIVEN `--no-default-features --features rustls-ring,carddav`
- WHEN `cargo tree -e normal -i rusqlite` runs
- THEN no package matches

### Requirement: Every backend builds alone
clippy `--all-targets -D warnings` SHALL pass on the default build, all features, `rustls-ring` with no backend, `native-tls,jmap`, `rustls-ring,vdir,pimdir` and each backend alone with `rustls-ring`, checked in CI.

### Requirement: Live provider suites
tests/google.rs, tests/microsoft.rs and tests/fastmail.rs SHALL run the built binary against Google (People, CardDAV), Microsoft Graph and Fastmail (JMAP, CardDAV), ignored unless asked: `anonymous::` with a made-up token, `credentialed::` read-only on the test accounts with tokens the suite mints, in the cardamum repository once its secrets exist. CI SHALL run them weekly (Mondays 03:00 UTC, the family's slot for cardamum) and by hand (`gh workflow run tests.yml`), never on a push, and a release SHALL run them by hand on the commit it tags; the offline jobs run on every push. A token SHALL reach the binary through its environment only, and no assertion SHALL print a card's value.
