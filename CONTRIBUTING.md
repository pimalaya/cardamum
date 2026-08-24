# Contributing guide

Thank you for investing your time in contributing to Cardamum.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation in [src/main.rs](./src/main.rs): it is the architecture document of this crate;
4. the [cairn](./cairn) folder for the living specification, the in-flight proposals and the landed history, activated by [AGENTS.md](./AGENTS.md).

Everything below documents only what differs from the Pimalaya standards.

## Where changes belong

Cardamum owns no protocol code. It aggregates the backends, renders the results and reads the configuration, and everything on the wire belongs to the crate that speaks that protocol: [io-webdav](https://github.com/pimalaya/io-webdav) for CardDAV, [io-jmap](https://github.com/pimalaya/io-jmap), [io-msgraph](https://github.com/pimalaya/io-msgraph), [io-people](https://github.com/pimalaya/io-people), [io-vdir](https://github.com/pimalaya/io-vdir) and [io-pimdir](https://github.com/pimalaya/io-pimdir).

The cross-backend layer is owned here rather than by an aggregator crate: the shared types live in src/shared, and each backend maps them onto its protocol crate through its own backend.rs. A concept only one protocol has belongs to that protocol's command family, not to the shared API.

## Feature matrix

Each backend sits behind its own cargo feature, and exactly one TLS provider must be on. The default set is `carddav`, `jmap`, `msgraph`, `people`, `vdir` and `rustls-ring`; `pimdir` is opt-in.

| Feature       | What it pulls in                                                     |
|---------------|----------------------------------------------------------------------|
| `carddav`     | io-webdav: the CardDAV backend and its discovery                     |
| `jmap`        | io-jmap and the vcard-rs JSContact codec                             |
| `msgraph`     | io-msgraph: the Microsoft Graph contacts API                         |
| `people`      | io-people: the Google People API                                     |
| `vdir`        | io-vdir: the local vdir home                                         |
| `pimdir`      | io-pimdir and io-replica: the local pimdir store                     |
| `rustls-ring` | the default TLS provider                                             |
| `rustls-aws`  | Rustls with the aws-lc crypto provider                               |
| `native-tls`  | the platform TLS stack                                               |

The shared client is fully cfg-gated, so a change touching a feature gate or an import is built against a narrower set before it lands, the default one hiding what a single-backend build breaks:

```sh
cargo build
cargo build --no-default-features --features rustls-ring,carddav
cargo build --no-default-features --features rustls-ring,jmap
cargo build --no-default-features --features rustls-ring,vdir
cargo build --no-default-features --features rustls-ring,pimdir
```

## Testing against a real server

Providers disagree on what they advertise and on what they then honour, so a backend change is verified by hand against a real account, `cardamum -a <account> account check` first for the connection, then the commands it touches. The reports live in [cairn/spec/testing](./cairn/spec/testing), one file per provider: add yours there, and never paste a secret into one.
