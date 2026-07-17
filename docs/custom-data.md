# Custom-data stash

A vCard can carry properties no provider field models. To keep them across a round-trip through Microsoft Graph or Google People — instead of silently dropping them on the next write — the projection stashes the **remainder** (every line it neither manages nor mints, see [contacts-mapping.md](contacts-mapping.md)) verbatim in a provider custom-data slot, and splices it back on read.

## Where the remainder lives

- **Google People**: a `clientData` entry keyed `cardamum.vcard` (`CLIENT_DATA_KEY`).
- **Microsoft Graph**: a `singleValueExtendedProperty` named `cardamum-vcard` (a `String {c8e5e5cf-…}` extended-property id).

On read, the projection reunites the managed fields, the minted read-only `X-*` vendor properties, and the stashed remainder into one vCard. On write, it recomputes the remainder from the incoming vCard, so the stash never drifts.

## Line-size limit

Lines longer than `MAX_STASH_LINE` (8 KiB — in practice base64 `PHOTO` blobs) are **not** stashed server-side: they stay only in the local document of record, so an oversized inline photo never risks the whole write against an undocumented provider size limit.

## Minted vendor properties

Provider-scoped fields that carry no meaning outside their account ride the vCard as read-only vendor properties (`X-GOOGLE-EXTERNAL-ID`, `X-GOOGLE-MISC-KEYWORD`, `X-GOOGLE-LOCATION`, `X-MSGRAPH-*`). They are minted on read and consumed (dropped) on write — the server value is authoritative — so they are neither managed nor part of the remainder.
