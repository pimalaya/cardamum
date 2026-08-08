---
cairn: change
change: lib-version-bump
---

# Delta

## ADDED Requirements

### Requirement: Table rendering is configurable
The `table.preset` option SHALL accept a `comfy-table` v7 positional preset string, one character per component, mapped onto the v8 typed style builder by shared/table.rs. Keeping the v7 spelling means an existing configuration stays valid across the v8 upgrade. A character position left out, or set to a space, draws nothing.

### Requirement: Card ids are backend-native and verbatim
A card's shared `id` SHALL be the backend's own identifier, used verbatim end to end. No adapter SHALL add or strip a file extension when addressing a resource. CardDAV names a new card `<uuid>.vcf` itself and passes the whole name through, since the server owns the resource name from then on.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
