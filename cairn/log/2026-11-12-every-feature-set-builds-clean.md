---
cairn: log
change: every-feature-set-builds-clean
date: 2026-11-12
---

# Every feature set builds clean

clippy `--all-targets -D warnings` passes on the default build, all features, `rustls-ring` alone (26 errors before: the unified client's dead arms and unread arguments, `account check` and `account list` locals no backend reads, `window` in a test build without the backends using it), `native-tls,jmap`, `rustls-ring,vdir,pimdir` and each of the six backends alone. A vdir-only build passes its 23 unit tests and the 8 first-run tests. .github/workflows/tests.yml runs the matrix in a `features` job, as Himalaya's does.

Capabilities moved: packaging (added "Feature-gated backends", "Every backend builds alone").
