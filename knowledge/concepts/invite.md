---
type: Domain Concept
title: Invite
description: Code starting with papo1 that carries the room secret plus up to four endpoint ids to dial; whoever holds it is a member.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/src/room.rs
tags: [papo, room, onboarding, security]
timestamp: 2026-10-02T00:00:00Z
---

# Invite

An invite is how a newcomer enters a [room](/concepts/room.md). It contains the room secret and a few endpoint ids of current members that the newcomer dials first. Endpoint ids are enough to connect because iroh resolves them to addresses through n0 DNS/pkarr and relays (see [iroh](/dependencies/iroh.md)). Byte layout: [invite encoding](/protocol/invite-encoding.md).

Because it carries the secret, an invite must be shared over a private channel. Anyone holding it can read and inject messages.

## Where invites come from

* `papo new --name <me>` creates a room and prints an invite containing only the creator's endpoint id.
* `papo invite` prints an invite with the caller's endpoint id plus up to three known peers, most recently seen first, so the newcomer can get in even when the caller is offline.
* `papo join <invite> --name <me>` stores the secret in a new [profile](/concepts/profile.md) and saves the invite's endpoint ids (minus its own) as known peers with unknown names. The MCP server then dials them (see [self-dial gotcha](/gotchas/gossip-pending-dial.md)).

Decoding trims whitespace and is case-insensitive, so invites survive chat apps that change case or add line breaks around them. Truncated or non-base32 invites are rejected with an error that hints at copy problems.
