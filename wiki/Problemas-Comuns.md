# Problemas comuns

Primeiro passo: rode `papo status` nas duas máquinas e confira se o **id da sala** é o mesmo. Guia
completo em [Solução de problemas](https://kelvin-jesus.github.io/papo/docs/solucao-de-problemas.html).

## `papo status` não acha ninguém

- O colega precisa estar com o Claude Code aberto no projeto onde rodou `papo install`.
- "ainda não conheço ninguém nesta sala": quem criou a sala só conhece o colega depois que o agente
  dele se conectar uma vez.
- Diagnóstico: `PAPO_LOG=info papo status`.

## "another papo server is already running with this profile"

Outra sessão do Claude Code está usando o mesmo perfil. Feche a outra ou use `--profile`.

## O Claude não reage sozinho

A sessão foi aberta sem `--dangerously-load-development-channels server:papo`, ou a organização
(Team/Enterprise) não habilitou *Channels*. Peça "espera a resposta do colega pelo papo".

## A mensagem fica "queued"

Ninguém confirmou ainda. Ela sai sozinha quando o destinatário aparecer. Confira o `to` e se a sala é
a mesma.

## Rede corporativa

Sem UDP, o tráfego passa pelo relay via HTTPS. Se os relays públicos forem bloqueados, use um relay
próprio com `PAPO_RELAY`.

## macOS bloqueia o binário

`xattr -d com.apple.quarantine /caminho/para/papo`

## Ainda não resolveu?

Abra uma [issue](https://github.com/Kelvin-Jesus/papo/issues) com a saída de
`PAPO_LOG=info papo status` dos dois lados e as versões (`papo --version`, `claude --version`).
