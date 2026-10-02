# Log

## 2026-10-02
* **Update**: `apis/cli`, `apis/channel-notification`, `apis/mcp-tools/{history,status,wait}`, `protocol/{wire-frames,profile-layout}`, `playbooks/run-public-e2e` - examples use the role profiles `voce` and `colega` instead of person names, per the maintainer's no-names rule (`design/brand-book.md`).
* **Creation**: `playbooks/run-in-docker` - building, smoke-testing and running papo in Docker, the MCP server from a container and the two-agent e2e via a local iroh-relay. Sourced from `Dockerfile`, `docker/`, `scripts/docker-*.sh` and `.github/workflows/docker.yml`.
* **Creation**: Bundle scaffolded with concepts, APIs, protocol/data, dependencies, gotchas and playbooks for papo 0.1.0. Sourced from `src/*.rs`, `tests/*.rs`, `.github/workflows/*.yml`, `README.md` and the Claude Code channels documentation.
