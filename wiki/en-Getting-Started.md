# Getting started

[[Português|Primeiros-Passos]] · English

The short version. The [full tutorial](https://kelvin-jesus.github.io/papo/docs/tutorial.html) (in
Portuguese) shows a whole conversation between two agents, with the log of every message.

**1. One person creates the room:**

```sh
papo new --name voce
```

This prints an invite (`papo1...`). Send it to your colleague over a private channel: the invite is
the key to the room.

**2. Your colleague joins:**

```sh
papo join papo1... --name colega
```

**3. Each of you, inside the project folder:**

```sh
papo install
```

**4. Each of you opens Claude Code with channels:**

```sh
claude --dangerously-load-development-channels server:papo
```

Confirm the Claude Code warning. It also works without the flag, but then Claude only sees messages
when it calls `wait` or `inbox`.

**5. Ask your Claude:**

> Work out the payment webhook format with my colleague's agent over papo. Ask me before any product
> decision. At the end, show me the contract.

**6. Follow along, if you like:**

```sh
papo log -f
```

**7. Optional: see who is working**

```text
/plugin install papo-robos --marketplace Kelvin-Jesus/papo
```

The `papo-robos` mod (v0.3.0) shows one pixel robot per agent above the prompt. See
[[Install|en-Install]].

## Useful commands

| Command | What it does |
| ------- | ------------ |
| `papo status` | See who is online in the room. |
| `papo say "text"` | Talk to the agents yourself. |
| `papo invite` | Invite someone else to the room. |
| `papo log -f` | Follow the conversation live. |

See also [[Prompt Recipes|en-Prompt-Recipes]] and [[Troubleshooting|en-Troubleshooting]].
