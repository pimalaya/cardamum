---
cairn: log
change: pimdir-write-guards
landed: 2026-08-09
---

# pimdir: a rename that renamed the wrong thing, and three writes that never looked first

The 2026-08-09 pimdir run found four defects, all on the shared arm, all now fixed and re-verified against both a bare store and one seeded to look like a sync engine populated it.

**The rename renamed the identifier.** io-pimdir's `rename_collection(collection, new_id)` renames the id, as its parameter name says, and the backend was calling it with the display name. So `addressbook update -k contacts -n "Contacts"` left `{"id": "Contacts", "name": "contacts"}`, every card under an id nobody typed, and the account's `addressbook.default` matching nothing, so a plain `card list` reported zero contacts.

There was no local fix that kept the flag: io-pimdir exposes no display-name setter, the name being a column the sync writes from the server, and a rename this backend cannot push is exactly what it refuses to do elsewhere. `update_addressbook` now always fails, joining `delete_addressbook`, and says why. The [backends](../spec/backends.md) requirement that used to promise a rename says the opposite now.

**Three commands wrote or read through without looking.** `card list -k <unknown>` printed an empty table and exited 0, so an unknown addressbook was indistinguishable from an empty one; `card create -k <unknown>` created the collection, so a typo invented an addressbook and hid the card in it. Both went through io-pimdir seams that are permissive by design: the write seam creates a collection on demand, the read seam answers an unknown one with an empty page. A `known_collection` guard now resolves the addressbook before `list_cards`, `get_card` and `create_card` act, and fails with "Addressbook `x` not found".

**A missing body listed as a blank row.** The listing read the blob with `unwrap_or_default()`, so an item claiming an object whose file is gone rendered with an empty name, while `get_card` on the same item correctly reported "Body blob missing". The listing now falls back to the summary preview, exactly as an unhydrated card does, and logs a warning, since a store missing a live blob is inconsistent and a blank row says nothing.

The paths the same run verified are untouched: an unhydrated card still previews from its `v: 1` summary and still refuses to be read, and a write against a synced collection still stages as a dirty placement, a tombstone or a baseless create, all three re-checked afterwards.
