# Contributing guide

Thank you for investing your time in contributing to Cardamum CLI.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation in [src/main.rs](./src/main.rs): it is the architecture document of this crate;
4. the [cairn/](./cairn) folder for the living spec, the in-flight proposals and the dated history (the Cairn convention: spec/, changes/, log/), activated by [AGENTS.md](./AGENTS.md).

This document stays operational; the design lives in the src/main.rs header and the behavioural truth in [cairn/spec](./cairn/spec).

## Development environment

The environment is managed by [Nix](https://nixos.org/download.html). `nix develop` spawns a shell with the right toolchain; every cargo command below assumes it (or prefix them with `nix develop --command`).

Without Nix, install a recent stable toolchain via [rustup](https://rust-lang.github.io/rustup/) (`rustup update`); the crate needs Rust matching the `rust-version` in [Cargo.toml](./Cargo.toml).

## Build

Cardamum is a CLI application built on the Pimalaya sans-I/O libraries. Backends and the TLS provider are cargo features; the default set is `rustls-ring`, `carddav` and `vdir`:

```sh
cargo build                                                   # default features
cargo build --no-default-features --features vdir,rustls-ring # vdir only, no CardDAV
cargo build --release
```

`carddav` and `vdir` toggle the two backends; exactly one TLS provider must be on (`rustls-ring`, `rustls-aws`, or `native-tls`). When touching feature gates or imports, build at least the default set and a single-backend set, so no backend-only code leaks across a disabled gate.

## Lint, test, audit

```sh
cargo test                       # unit + doc tests
cargo clippy --all-targets       # keep clean for every feature set you touch
cargo fmt                        # CI checks `cargo fmt --check`
cargo deny check                 # licenses, advisories, sources
```

Before opening a PR, make sure `cargo test`, `cargo clippy`, `cargo fmt --check` and `cargo deny check` pass.

## Commit style

Cardamum CLI follows the [conventional commits specification](https://www.conventionalcommits.org/en/v1.0.0/#summary). Keep the subject imperative and scoped; describe the *why* in the body when it is not obvious.
