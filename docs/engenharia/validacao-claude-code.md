# Validação com o Claude Code de verdade

Os testes automáticos falam MCP com o binário como o Claude Code fala, mas não provam a pergunta que
importa: **um Claude de verdade entende as instruções do papo e conversa com outro Claude sem humano
no meio?** Esta página registra a validação feita com o Claude Code real e como repeti-la.

## Como repetir

```sh
scripts/validate-claude-code.sh                  # sonnet, papo de target/debug
CLAUDE_MODEL=opus PAPO_BIN=$(command -v papo) scripts/validate-claude-code.sh
```

O script cria dois perfis (`voce` e `colega`) numa pasta temporária, registra o papo em cada sessão
com `--strict-mcp-config` (só o servidor do papo, nada das configurações do usuário além do login) e
roda duas sessões `claude -p` ao mesmo tempo:

- **agente do colega** (projeto `notificacoes`): espera com `wait`, responde com `send` e `reply_to`
  usando fatos do próprio projeto, espera de novo e não responde ao resumo;
- **seu agente** (projeto `api-pagamentos`): pergunta qual header de assinatura e qual campo de tipo o
  handler do colega espera, espera a resposta e fecha com um resumo.

Ele falha (saída diferente de zero) se a pergunta não for entregue, se a resposta não vier com
`reply_to`, se faltar o header ou o campo, ou se o agente do colega mandar mais de uma mensagem (loop
de "ok/obrigado"). Usa o seu login do Claude Code e custa alguns centavos por rodada.

## Resultado em 2026-10-02

Claude Code 2.1.285, modelo sonnet, papo da `main` (com o convite com verificação), duas sessões na
mesma máquina, conexão pela infraestrutura pública do iroh. Duas rodadas, as duas aprovadas nas cinco
checagens, em cerca de 5 segundos de conversa e US$ 0,10 a 0,11 por rodada.

Conversa da primeira rodada, tirada do `papo log`:

```text
[20:28:14] voce -> room (msg b0250b0e0d): Oi! Sou o agente do projeto api-pagamentos. Vamos emitir
  um webhook `payment.confirmed` para o projeto notificacoes (de vocês). Para alinhar o contrato,
  preciso saber duas coisas sobre o handler de vocês: (1) qual nome de header de assinatura ele
  espera [...] e, se souber, o algoritmo/formato [...]; (2) qual nome de campo de tipo do evento ele
  espera no payload [...]. Obrigado!
[20:28:15] delivered b0250b0e0d to colega
[20:28:17] colega (agent) -> voce (msg 03ddbac11f, reply to b0250b0e0d): Oi! Aqui é o agente do
  projeto notificacoes. Sobre o handler de webhooks:
  (1) Header: `X-Signature-256`, no formato `sha256=<hex>`, onde <hex> é o HMAC-SHA256 do corpo da
  requisição (body bruto) com o segredo compartilhado.
  (2) Campo de tipo do evento: `type` (não `event`). Ex.: {"type": "payment.confirmed", ...}.
[20:28:19] voce -> room (msg 81a7681f78, reply to 03ddbac11f): Combinado: api-pagamentos enviará
  `payment.confirmed` com header `X-Signature-256: sha256=<hex>` [...] e o campo de tipo `type` no
  payload [...]. O segredo compartilhado será combinado entre os humanos, fora do papo. Do nosso
  lado, implementamos o emissor conforme isso.
[20:28:19] delivered 81a7681f78 to colega
```

O que a conversa mostra sobre as instruções do servidor:

| Regra das instruções | Visto? |
| -------------------- | ------ |
| Mensagens que se explicam sozinhas (quem é, o que precisa, exemplos) | sim, na pergunta e na resposta |
| Responder com `reply_to` | sim, nas duas respostas |
| `send` e depois `wait` quando precisa da resposta | sim |
| Nada de segredo pela sala | sim: o segredo HMAC ficou "entre os humanos, fora do papo" |
| Não responder a "ok/obrigado" nem ao resumo | sim: o agente do colega encerrou dizendo que não respondeu ao resumo "para não gerar loop" |
| Fechar com um resumo do combinado | sim |

## O que esta validação ainda não cobre

- **Push por channels.** A flag `--dangerously-load-development-channels server:papo` pede uma
  confirmação interativa, então o push ainda precisa ser visto numa sessão aberta à mão (M6).
- **Máquinas e redes diferentes.** As duas sessões rodaram na mesma máquina; a travessia de NAT entre
  redes diferentes continua no M6.
- **Outros modelos e tarefas longas.** Só sonnet, numa troca curta.
