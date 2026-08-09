---
cairn: change
id: vdir-write-guards
status: landed
created: 2026-08-09
---

# vdir: writing through to the filesystem without looking first

The 2026-08-09 vdir run ([vdir-local.md](../../spec/testing/vdir-local.md)) found the specific surface clean and six defects on the shared one. Four are the local twins of what the network backends were fixed for the same day, and two were carried over from July.

**V5: `card update <absent-id>` creates the file.** The shared update calls `store_item` with whatever id it is given, so `card update -k contacts absent-id …` writes `absent-id.vcf` and reports success. The specific `vdir item update` on the same id fails properly, so the shared path is the outlier: this is CardDAV's F5 on a filesystem.

**V6, V7, V8: three commands that never resolve the collection.** `card create -k <unknown>` creates the collection, so a typo invents an addressbook and hides the card in it. `card list -k <unknown>` prints an empty table and exits 0. `addressbook delete -k <unknown>` surfaces `No such file or directory (os error 2)`. The specific arm already resolves a collection through `VdirClient::collection_path`, which is what gives it the friendly "Collection `<name>` not found"; the shared arm builds the path and hopes.

**V1: clearing collection metadata is a silent no-op.** `addressbook update -k <id> -d ""` reports success and leaves the `description` file in place. io-vdir's collection update writes a file per present, non-empty value and has no removal path, exactly like io-webdav's PROPPATCH before it learned `DAV:remove` earlier today.

**V2: `card update --if-match` is silently ignored.** vdir has no ETag, which is fine, but msgraph and jmap bail on the same flag rather than accept and drop it.

## What changes

- io-vdir's collection update writes the collection it is handed: a present value is written, an absent or empty one has its metadata file removed. The struct is a desired state rather than a patch, so this is what its signature already promised.
- The shared `addressbook_path` resolves the collection instead of assuming it, so `card list`, `card create`, `addressbook delete` and the rest fail with "Addressbook `<id>` not found".
- `update_card` reads the item first, so an unknown id fails instead of creating a file.
- `update_card` bails on `--if-match`, matching msgraph and jmap.

## Note for other io-vdir consumers

The collection update is a behaviour change: a caller passing a partially-filled `VdirCollection` used to keep the fields it left unset and now clears them. Cardamum merges its patch over the current collection before calling, so it is unaffected; calendula does the same on the calendar side and wants checking when it next picks the crate up.
