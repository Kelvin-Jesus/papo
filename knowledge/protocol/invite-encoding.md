---
type: Data Format
title: Invite encoding
description: An invite is "papo1" followed by lowercase unpadded base32 of the 32-byte room secret, up to four 32-byte endpoint ids and a 4-byte integrity check.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/room.rs
tags: [papo, protocol, onboarding]
timestamp: 2026-10-02T00:00:00Z
---

# Invite encoding

# Schema

```text
payload = secret[32] || endpoint_id[32] * n,  n <= 4
check   = blake3::derive_key("papo v1 invite check", payload)[..4]
invite  = "papo1" + lowercase( BASE32_NOPAD( payload || check ) )
```

* Decoding trims whitespace, uppercases before base32 decoding, requires `len - 4` to be a positive multiple of 32 bytes with at least the secret, and verifies the check ("invite is damaged or incomplete" otherwise).
* The check exists because a copy-paste cut at a 32-byte boundary used to decode as a valid invite with fewer entry points, silently. A property test (`tests/properties.rs`, `truncated_invites_are_rejected`) found it; it also catches most typos.
* Each 32-byte chunk after the secret must be a valid iroh public key.
* With one endpoint id an invite is 114 characters (`papo1` + 109 base32 chars for 68 bytes).
* Changing this format invalidates every invite already shared; the check was added before the first release.

Concept and lifecycle: [invite](/concepts/invite.md).
