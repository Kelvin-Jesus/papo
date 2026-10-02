---
type: Playbook
title: Run and test papo in Docker
description: Build the static papo image, smoke-test it, run the MCP server from a container and run the two-agent e2e through a local relay.
resource: https://github.com/Kelvin-Jesus/papo/blob/main/docs/guias/docker.md
tags: [papo, docker, testing, networking]
timestamp: 2026-10-02T00:00:00Z
---

# Run and test papo in Docker

```sh
docker build -t papo:dev .                 # static musl binary on distroless, non-root, PAPO_HOME=/data
docker build --target test .               # cargo test --locked inside the builder image
scripts/docker-smoke.sh                    # --version, new into a volume, MCP initialize/tools/list/status, lock
scripts/docker-e2e.sh                      # two agents + docker/relay (iroh-relay in dev mode)
scripts/docker-e2e.sh --public             # same, through the public n0 relays
```

- The image is about 8 MB compressed; the binary inside is 16.7 MB. arm64 builds cross-compile with zig.
- MCP from a container: `claude mcp add papo -- docker run -i --rm -v papo-data:/data ghcr.io/kelvin-jesus/papo mcp`.
  One container per profile (the profile lock refuses a second one). Set `--about` on `new`/`join`,
  because the default `about` is the container's working directory.
- Both e2e modes still need the internet for the n0 DNS/pkarr lookup that finds peers by endpoint id.
  A fully offline run would need a code change (a static address list).
- `docker/relay/` is an `iroh-relay` in dev mode for local use only; point both sides at it with
  `PAPO_RELAY=http://relay:3340`.
- CI: `.github/workflows/docker.yml` builds and smoke-tests on PRs and main, runs the e2e as informative,
  and on `v*` tags pushes multi-arch images to `ghcr.io/kelvin-jesus/papo`.
