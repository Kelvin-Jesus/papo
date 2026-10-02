# Knowledge bundle: Open Knowledge Format (OKF)

Asset-level knowledge about papo lives in the OKF v0.1 bundle at `knowledge/`: plain markdown files with YAML frontmatter, cross-linked into a concept graph. It complements `CONTEXT.md` (domain language) and `docs/adr/` (decisions). The human docs book in `docs/` is for users; the bundle is for agents and maintainers.

## Layout

```
knowledge/
├── index.md         ← bundle root (only index.md allowed frontmatter: okf_version)
├── log.md           ← change log, newest first
├── concepts/        ← room, invite, profile, members, delivery, push vs pull
├── apis/            ← MCP server, mcp-tools/<tool>.md, channel notification, CLI, env vars
├── protocol/        ← wire frames, sealing, invite encoding, profile layout
├── dependencies/    ← iroh, iroh-gossip, Claude Code channels
├── gotchas/         ← failure modes and the rules that prevent them
└── playbooks/       ← release, debug connectivity, add a tool, rotate a room, public e2e
```

## Consuming (read)

- Start at `knowledge/index.md` and follow the section `index.md` files (progressive disclosure).
- Before touching networking read `gotchas/`; before touching the MCP surface read `apis/`.
- Tolerate gaps: missing fields and broken links mark knowledge not yet written.

## Producing (write)

When you learn a durable fact (a new tool, a changed frame, a dependency quirk, an operational procedure):

1. Create or update `knowledge/<section>/<kebab-slug>.md` (never name it `index.md` or `log.md`).
2. Frontmatter: `type` is required; add `title`, `description`, `resource` (GitHub URL of the source file or external docs), `tags`, `timestamp` (ISO 8601).
3. Use bundle-absolute links (`/protocol/wire-frames.md`), conventional headings `# Schema`, `# Examples`, `# Citations`.
4. Add the concept to its directory `index.md` (`* [Title](/path.md) - description`).
5. Add a dated entry at the top of `knowledge/log.md` (`## YYYY-MM-DD`, `* **Creation**|**Update**|**Deprecation**: ...`).
6. Validate: every non-reserved file has frontmatter with a non-empty `type`; check links resolve. The `okf-open-knowledge-format` skill ships a validator script.

Never invent facts: source them from the code, tests, workflows or cited documentation. English only.

The workspace-wide bundle at `/home/kj/Developer/knowledge/papo/` (outside this repo) only points here.
