---
cairn: change
id: every-feature-set-builds-clean
status: landed
created: 2026-11-12
---

# Every feature set builds clean

## Why

Sixth change of the-family-standards. A packager picks the backends it ships, and AGENTS.md asks that each compile alone; nothing checked it, and the build with no backend failed clippy on dead code.

## What

- src/shared/client.rs: a build without any backend keeps the unified client, the command tree being the same in every build, and allows its dead arms and unread arguments there, the reason given once in the module header; `window` compiles for the pimdir tests rather than any test build. src/account/check.rs and src/account/list.rs allow what no backend reads in that build.
- .github/workflows/tests.yml gains Himalaya's `features` job: clippy `--all-targets -D warnings` over the default build, all features, `rustls-ring` alone, `native-tls,jmap`, `rustls-ring,vdir,pimdir` and each backend alone with `rustls-ring`, then the tests of a vdir-only build.

## Scope / non-goals

No behaviour moves: the commands, outputs and accepted configurations of every build are the same.
