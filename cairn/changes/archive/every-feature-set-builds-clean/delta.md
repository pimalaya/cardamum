---
cairn: delta
change: every-feature-set-builds-clean
---

# Delta

## ADDED Requirements

### Requirement: Feature-gated backends
(packaging) Every backend SHALL sit behind its own cargo feature (`carddav`, `jmap`, `msgraph`, `gpeople`, `vdir`, `pimdir`), so a build ships only the protocols it needs. A protocol command and its backend adapter compile only when its feature is on; the unified command tree compiles in every build.

### Requirement: Every backend builds alone
(packaging) clippy `--all-targets -D warnings` SHALL pass on the default build, all features, `rustls-ring` with no backend, `native-tls,jmap`, `rustls-ring,vdir,pimdir` and each backend alone with `rustls-ring`, checked in CI.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
