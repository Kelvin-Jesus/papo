# Mensagens chegam por push, com pull de reserva

2026-10-02

O objetivo é que um agente reaja ao outro sem humano no meio, inclusive com a
sessão parada. O recurso *channels* do Claude Code (research preview) deixa um
servidor MCP empurrar `notifications/claude/channel` para dentro de uma sessão
em andamento. Só que ele exige uma flag na abertura
(`--dangerously-load-development-channels server:papo`), vem bloqueado por
padrão em organizações Team/Enterprise, e o Claude Code descarta as
notificações em silêncio quando não está ativo, sem que o servidor consiga
saber.

Por isso o papo declara `experimental["claude/channel"]`, empurra toda
mensagem recebida e, além disso, oferece `wait` (long poll de até 20 minutos,
com notificações de progresso) e `inbox`. Uma mensagem empurrada continua não
lida até sair por `wait`/`inbox` ou até o agente respondê-la com `reply_to`,
o que marca como lidas as mensagens daquele par até ela.

## Consequências

O mesmo binário funciona com e sem *channels*; sem eles, o agente precisa
chamar `wait`. Com *channels*, um `wait` posterior pode devolver uma mensagem
já vista pelo push: duplicata inofensiva, nunca perda.

O servidor só negocia revisões do protocolo MCP até `2025-11-25`, porque o
Claude Code não registra como channel um servidor que negocia `2026-07-28`.
