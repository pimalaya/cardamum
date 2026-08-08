---
cairn: log
change: pimdir-local-backend
landed: 2026-08-09
---

# pimdir local backend

Added a sixth backend: Cardamum over a local pimdir store, the SQLite-indexed, content-addressed offline cache the sync engine populates. It is the contacts twin of Himalaya's pimdir cache backend, and it reads and writes the same store a Neverest sync drives.

`src/pimdir/` holds four files. client.rs opens the store as a source, auto-detected from `distinct_sources` when `pimdir.source` is unset, shell-expanding the root first (opening it raw would create an empty store at a literal `./~/…` and silently list nothing). hash.rs is the 128-bit FNV-1a content digest, ported so a card Cardamum adds deduplicates against the same card a sync stored. card.rs derives the link id (`uid:<UID>`, else `hash:<fnv1a64>`), the `v: 1` summary and the sort key exactly as Neverest's `text/vcard` kind does, using the same small scanner rather than a vCard parser so a property it does not understand cannot be lost. backend.rs adapts the nine shared operations.

The cache semantics drove two decisions. A card whose body is not local still lists, projected from its stored summary into a minimal preview vCard (`UID`, `FN`, `EMAIL`), because the shared `Card` conflates metadata and document and a blank row would make an unsynced store useless; `get_card` refuses such a card outright, so the preview can never be mistaken for the document of record. Writes stage io-replica mutations through the mutate seam rather than touching SQL, `Edit` being what makes contacts workable at all, and a placement with no sync base fails loudly rather than staging a change no sync would carry.

Three limits are explicit rather than emulated: `delete_addressbook` always fails (io-pimdir exposes no collection removal and io-replica has no collection-level mutation), `update_addressbook` honours a rename and rejects description or colour, and `update_card` ignores `--if-match` because the engine reconciles against the base body it recorded at sync time.

The wizard's local-path branch now detects the store kind from on-disk markers (`pimdir.db` for pimdir, a subdirectory holding a `.vcf` for vdir) and prompts only when both are compiled in and detection is inconclusive.

Not yet exercised against a real synced store: the read and write paths both need one, and that run is the outstanding item in cairn/spec/testing/README.md.

Capabilities moved: backends (six new requirements), wizard (local detection), config (the `[pimdir]` block).
