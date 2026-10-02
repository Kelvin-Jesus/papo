# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root: the ubiquitous language (room, invite, profile, member, agent, human, ephemeral node, neighbor, envelope, ack, outbox, inbox, push, pull).
- **`docs/adr/`**: read the ADRs that touch the area you are about to work in. `docs/adr/README.md` is the index.

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill creates them lazily when terms or decisions actually get resolved.

## File structure

Single-context repo:

```
/
├── CONTEXT.md
├── docs/adr/
│   ├── README.md
│   ├── 0001-o-transporte-e-iroh-com-gossip.md
│   └── ...
├── knowledge/          ← asset-level knowledge (OKF), see docs/agents/knowledge.md
└── src/
```

ADRs are written in Portuguese, one decision per file, with the decision as the title (`NNNN-a-decisao-em-kebab-case.md`), followed by the reasoning and a `Consequências` section. New ADRs take the next number and are added to `docs/adr/README.md` and `docs/SUMMARY.md`.

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Don't drift to synonyms the glossary avoids (for example, say "member" or "peer", not "client"; "room", not "channel", because channel means the Claude Code feature).

If the concept you need isn't in the glossary yet, that's a signal: either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0003 (o papo disca os pares antes do gossip), but worth reopening because..._
