---
type: External Library
title: iroh 1.3
description: QUIC networking library papo uses for endpoints addressed by public key, hole punching, n0 DNS/pkarr address lookup and relay fallback.
resource: https://docs.rs/iroh/1.3.0
tags: [papo, networking, dependency]
timestamp: 2026-10-02T00:00:00Z
---

# iroh 1.3

## How papo uses it

* Production endpoints: `Endpoint::builder(presets::N0).secret_key(<profile key>).bind()` in `src/net.rs`. The N0 preset adds `PkarrPublisher::n0_dns()` (publishes the endpoint's relay URL and addresses to `https://dns.iroh.link/pkarr`), `PkarrResolver::n0_dns()`, `DnsAddressLookup::n0_dns()` and the default n0 relays (`use1-1`, `usw1-1`, `euc1-1`, `aps1-1` under `relay.n0.iroh.link`).
* `PAPO_RELAY=<url>` switches to `RelayMode::Custom` for a self-hosted `iroh-relay`.
* A `Router` accepts the gossip ALPN and hands connections to [iroh-gossip](/dependencies/iroh-gossip.md). `Router::shutdown` closes the endpoint so peers notice the departure quickly.
* Endpoint id = public key of `secret.key`; `EndpointId::fmt_short()` (10 hex chars) is what logs and `status` show.
* An endpoint is dialable by id only after it has a home relay and has published to pkarr, which takes about one second after start. Dialing earlier fails with `No addressing information available` (see [pending dial gotcha](/gotchas/gossip-pending-dial.md)).

## Tests

`tests/node.rs` uses the `test-utils` feature (dev-dependency only): `iroh::test_utils::run_relay_server()`, `presets::Minimal`, `RelayMode::Custom`, `CaTlsConfig::insecure_skip_verify()` and a shared `MemoryLookup`, so integration tests never touch the internet.

The public n0 relays are free and rate limited; heavy use should self-host a relay.

# Citations

[1] [iroh](https://iroh.computer)
[2] [iroh 1.3.0 API docs](https://docs.rs/iroh/1.3.0)
