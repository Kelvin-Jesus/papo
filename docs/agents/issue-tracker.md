# Issue tracker: GitHub Issues

Issues for this repo live in GitHub Issues on `Kelvin-Jesus/papo`, managed with the `gh` CLI.

## Conventions

- One issue per problem or feature. Title in Portuguese, imperative or descriptive (`Fila não reenvia após reconectar`).
- Body: what happened, how to reproduce (commands, `papo --version`, OS), expected behavior, and relevant `PAPO_LOG` output with room secrets and invites removed.
- Labels: `bug`, `enhancement`, `documentation`, `question` (GitHub defaults). Add a label only if it already exists in the repo.
- Reference issues from commits and PRs with `#<n>`; close them with `Closes #<n>` in the PR body.

## When a skill says "publish to the issue tracker"

```sh
gh issue create -R Kelvin-Jesus/papo --title "<title>" --body-file <file.md> [--label bug]
```

## When a skill says "fetch the relevant ticket"

```sh
gh issue view <n> -R Kelvin-Jesus/papo --comments
```

## Other operations

- List open issues: `gh issue list -R Kelvin-Jesus/papo`
- Comment: `gh issue comment <n> -R Kelvin-Jesus/papo --body "<text>"`
- Close: `gh issue close <n> -R Kelvin-Jesus/papo --comment "<why>"`

Never paste invites (`papo1...`), `profile.json` or `secret.key` contents into issues: the repo is public.
