---
type: Data Format
title: Invite encoding
description: An invite is "papo1" followed by lowercase unpadded base32 of the 32-byte room secret and up to four 32-byte endpoint ids.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/room.rs
tags: [papo, protocol, onboarding]
timestamp: 2026-10-02T00:00:00Z
---

# Invite encoding

# Schema

```text
"papo1" + lowercase( BASE32_NOPAD( secret[32] || endpoint_id[32] * n ) ),  n <= 4
```

* Decoding trims whitespace, uppercases before base32 decoding, and requires a total length that is a positive multiple of 32 bytes with at least the secret.
* Each 32-byte chunk after the secret must be a valid iroh public key.
* With one endpoint id an invite is 108 characters (`papo1` + 103 base32 chars for 64 bytes).

Concept and lifecycle: [invite](/concepts/invite.md).
