# Protocol and data

* [Wire frames](/protocol/wire-frames.md) - JSON frames hello, msg and ack broadcast over the room's gossip topic; size limits and ids.
* [Frame sealing](/protocol/sealing.md) - XChaCha20-Poly1305 with a blake3-derived room key, random nonce prefix and fixed AAD.
* [Invite encoding](/protocol/invite-encoding.md) - papo1 prefix plus lowercase unpadded base32 of the secret and endpoint ids.
* [Profile layout](/protocol/profile-layout.md) - Files under $PAPO_HOME/profiles/<profile>/ and how each is written.
