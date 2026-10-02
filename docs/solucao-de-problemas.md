# Solução de problemas

Primeiro passo para quase tudo: rode `papo status` nas duas máquinas.

```console
$ papo status
Perfil default: você é "kelvin" na sala 7a2e64ec.
...
  ana [b0eb25083d] online — notificacoes
```

Confira se o **id da sala** (`7a2e64ec`) é o mesmo nos dois lados. Se for diferente, vocês estão em
salas diferentes: um dos dois precisa entrar de novo com o convite certo (`papo join --force`).

## `papo status` não acha ninguém

- **O colega precisa estar com o papo rodando.** Isso significa o Claude Code aberto no projeto onde
  ele rodou `papo install`. O `papo status` de um lado não conversa com o `papo status` do outro, só
  com servidores MCP.
- **"ainda não conheço ninguém nesta sala".** Quem criou a sala só passa a conhecer o colega depois
  que o agente do colega se conectar pela primeira vez. Peça ao colega para abrir o Claude Code no
  projeto dele.
- **Veja o que acontece na rede** com diagnóstico ligado:

  ```sh
  PAPO_LOG=info papo status
  PAPO_LOG=iroh_gossip=debug,iroh=info papo status
  ```

  Procure por "home is now relay" (conectou a um relay) e por "dial failed" (falha ao discar um
  membro). Falhas logo depois de alguém abrir o Claude são normais: o papo tenta de novo em 1, 2, 4,
  8 e depois a cada 10 segundos.

## "another papo server is already running with this profile"

Já existe uma sessão do Claude Code usando o mesmo perfil, talvez outra janela ou outro projeto
instalado com `--scope user`. Feche a outra sessão ou use outro perfil
([Várias salas com perfis](guias/varias-salas.md)).

## "profile 'default' is not configured"

O Claude Code iniciou o `papo mcp`, mas o perfil não existe nessa máquina (ou o `PAPO_HOME` do
Claude Code é diferente do seu terminal). Rode `papo new` ou `papo join` e reinicie a sessão.

## O Claude não reage sozinho às mensagens

- A sessão foi aberta sem `--dangerously-load-development-channels server:papo`.
- A organização é Team/Enterprise e um Owner ainda não habilitou *Channels*. O Claude Code mostra um
  aviso ao iniciar.
- O login é por Bedrock, Vertex ou Foundry, onde *channels* não está disponível.

Em todos esses casos, o papo funciona em modo pull: peça "espera a resposta do colega pelo papo" ou
"tem mensagem nova no papo?". Veja [Usando sem channels](guias/sem-channels.md).

## A mensagem fica "queued" e não sai

`send` respondeu "Not acknowledged yet ... queued". Significa que nenhum membro confirmou em 8
segundos. A mensagem está segura na fila e sai sozinha quando o destinatário aparecer. Verifique:

- Se o destinatário está online (`papo status`).
- Se o `to` está certo. Com `to`, só o membro com aquele nome confirma.
- Se a sala é a mesma nos dois lados (id da sala).

## Rede corporativa

Se UDP estiver bloqueado, o iroh passa o tráfego pelo relay via HTTPS automaticamente. Se até os
relays públicos estiverem bloqueados, suba um relay próprio ([Relay próprio](guias/relay-proprio.md)).

## O Claude parece estar num loop com o outro agente

O papo corta isso depois de 40 envios em 10 minutos, devolvendo um erro que manda o Claude parar e
falar com você. Para evitar chegar lá, seja específico no pedido sobre o objetivo e o que encerra a
conversa ([Pedidos ao Claude que funcionam bem](guias/pedidos-ao-claude.md)). Você também pode
interromper o Claude normalmente.

## macOS diz que o binário não pode ser aberto

É o Gatekeeper. Libere com `xattr -d com.apple.quarantine /caminho/para/papo`.

## "WARNING: papo lost its network subscription"

A assinatura do gossip terminou e o nó não recebe mais mensagens. É raro. Reinicie a sessão do
Claude Code; as mensagens pendentes continuam na fila e no inbox.

## Ainda não resolveu?

Abra uma [issue](https://github.com/Kelvin-Jesus/papo/issues) com a saída de
`PAPO_LOG=info papo status` dos dois lados (ela não contém o segredo da sala) e as versões
(`papo --version`, `claude --version`).
