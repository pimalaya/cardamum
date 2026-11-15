# Contributing guide

Thank you for investing your time in contributing to Cardamum.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation in [src/main.rs](./src/main.rs): it is the architecture document of this crate;
4. the [cairn](./cairn) folder for the living specification, the in-flight proposals and the landed history, activated by [AGENTS.md](./AGENTS.md).

Everything below documents only what differs from the Pimalaya standards.

## Where changes belong

Cardamum owns no protocol code. It aggregates the backends, renders the results and reads the configuration, and everything on the wire belongs to the crate that speaks that protocol: [io-webdav](https://github.com/pimalaya/io-webdav) for CardDAV, [io-jmap](https://github.com/pimalaya/io-jmap), [io-msgraph](https://github.com/pimalaya/io-msgraph), [io-gpeople](https://github.com/pimalaya/io-gpeople), [io-vdir](https://github.com/pimalaya/io-vdir) and [io-pimdir](https://github.com/pimalaya/io-pimdir).

The cross-backend layer is owned here rather than by an aggregator crate: the shared types live in src/shared, and each backend maps them onto its protocol crate through its own backend.rs. A verb some backends serve with the same result joins the unified API, each backend declaring its level in src/capability.rs (`full`, `partial`, `server` or `none`); a concept only one protocol has, or one whose result differs, belongs to that protocol's command family.

The pimdir derivations (link id, summary, sort key, content hash) are io-pimdir's own and never reimplemented here: Neverest writes into the same store through the same crate, and a card staged by cardamum has to link and summarize exactly as one arriving through a sync.

A test follows the correct behaviour: never adjust production code to make one pass. A user-facing change lands in CHANGELOG.md too, a release-scoped roll-up over the Cairn log rather than a replacement for it.

## Feature matrix

Each backend sits behind its own cargo feature, and a network backend needs a TLS provider. The default set is every backend, `rustls-ring` and `vendored`.

| Feature       | What it pulls in                                                     |
|---------------|----------------------------------------------------------------------|
| `carddav`     | io-webdav: the CardDAV backend                                       |
| `jmap`        | io-jmap and the vcard-rs JSContact codec                             |
| `msgraph`     | io-msgraph: the Microsoft Graph contacts API                         |
| `gpeople`     | io-gpeople: the Google People API                                    |
| `vdir`        | io-vdir: the local vdir home                                         |
| `pimdir`      | io-pimdir: the local pimdir store                                    |
| `rustls-ring` | the default TLS provider                                             |
| `rustls-aws`  | Rustls with the aws-lc crypto provider                               |
| `native-tls`  | the platform TLS stack                                               |
| `vendored`    | SQLite and OpenSSL built from source rather than linked              |

The unified client is fully cfg-gated, so a change touching a feature gate or an import is checked against the feature matrix before it lands, the default build hiding what a single-backend build breaks. The CI `features` job runs clippy over every set, each backend alone included; locally:

```sh
cargo clippy --all-targets --no-default-features --features rustls-ring -- -D warnings
cargo clippy --all-targets --no-default-features --features rustls-ring,carddav -- -D warnings
cargo clippy --all-targets --no-default-features --features rustls-ring,pimdir -- -D warnings
```

## Testing against a real server

Providers disagree on what they advertise and on what they then honour. The live suites (tests/google.rs, tests/microsoft.rs, tests/fastmail.rs) run on every push: `cargo test --test google -- --ignored anonymous::` touches no account, `credentialed::` reads the test accounts and needs their secrets. A backend change beyond them is verified by hand against a real account, `cardamum -a <account> account check` first for the connection, then the commands it touches. The reports live in [cairn/spec/testing](./cairn/spec/testing), one file per provider: add yours there, and never paste a secret into one, nor print one in output, a log line or a test fixture.

## Cairn

This repository follows [Cairn](https://github.com/pimalaya/cairn): a change that affects behaviour is not done until the specification is updated and a log entry is written, and a landed change moves to cairn/changes/archive/; `sh cairn/verify.sh` checks the structure. See [AGENTS.md](./AGENTS.md) for the full stanza.
