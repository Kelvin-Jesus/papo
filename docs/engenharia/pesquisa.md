# Pesquisa: restrições, APIs e medições

Pesquisa feita em 2026-10-02 para o papo. Há três tipos de evidência aqui:

- **Medido**, marcado **[M]**: rodado na máquina de desenvolvimento (Linux x86_64, 12 núcleos, Rust
  1.98, Claude Code 2.1.285) ou no CI do GitHub.
- **Lido no código-fonte** das dependências, marcado **[C]**, com o arquivo citado.
- **De fontes primárias**, numeradas `[n]` e listadas no fim.

O que não foi confirmado está marcado **(não confirmado)**.

---

## 1. Claude Code: como um servidor MCP empurra mensagens para a sessão

### 1.1 Channels

O Claude Code tem um recurso em *research preview* chamado *channels*: um servidor MCP manda eventos
para dentro de uma sessão em andamento, e o Claude reage sem o usuário digitar [1][2].

| Ponto | O que a documentação diz |
| ----- | ------------------------ |
| Capability | `capabilities.experimental["claude/channel"] = {}` no resultado do `initialize`; sempre `{}` [1] |
| Notificação | método `notifications/claude/channel`, `params: { content: string, meta?: Record<string,string> }` [1] |
| Como chega ao Claude | `<channel source="<nome do servidor>" chave="valor">conteúdo</channel>`; `source` vem do nome configurado do servidor [1] |
| Chaves do `meta` | só letras, dígitos e `_`; chaves com hífen ou outros caracteres são descartadas em silêncio [1] |
| Vários eventos com o Claude ocupado | entregues juntos no turno seguinte, em ordem [1] |
| Abrir com um servidor que não é plugin | `claude --dangerously-load-development-channels server:<nome>`, com diálogo de confirmação; o servidor precisa estar configurado antes (`.mcp.json` ou configuração do usuário) [2] |
| `--channels` | aceita só plugins da lista permitida [2] |
| Autenticação | conta claude.ai ou chave do Console; não existe em Bedrock, Vertex ou Foundry [2] |
| Organizações | Pro e Max sem organização: liberado. Team e Enterprise: bloqueado até um Owner habilitar (admin do claude.ai ou `channelsEnabled: true` em configurações gerenciadas). Organizações do Console: liberado, a não ser que haja configurações gerenciadas [2] |
| Detecção | não há forma documentada de o servidor saber se channels está ativo; quando não está, o Claude Code descarta as notificações em silêncio, sem erro [1] |
| Instruções para o Claude | campo `instructions` do `initialize`, recomendado para dizer o que chega e como responder [1] |
| Respostas | ferramenta MCP comum; nada específico de channel [1] |
| Negociação de versão | com `MCP_PROTOCOL_NEGOTIATION=auto`, um servidor que negocia a revisão `2026-07-28` não é registrado como channel [3] |

**Consequências no papo:** a capability é declarada sempre; cada mensagem recebida vira uma
notificação com `from`, `msg_id`, `sender_kind` e, quando existem, `reply_to` e `to`; o servidor
negocia no máximo `2025-11-25`; e, como não dá para saber se o push chegou, existe o modo pull
([ADR 0002](../adr/0002-mensagens-chegam-por-push-com-pull-de-reserva.md)).

**Não confirmado:** a versão mínima do Claude Code com channels; se `claude mcp add --scope local`
basta para o `server:papo` da flag (a documentação cita `.mcp.json` e configuração do usuário).

A capability de *permission relay* (`claude/channel/permission`), que deixa o servidor aprovar pedidos
de permissão de ferramentas, existe mas o papo **não** a declara: aprovar uso de ferramenta pela sala
daria a outro agente poder sobre a sua máquina.

### 1.2 Timeouts de ferramentas

| Ponto | Fonte |
| ----- | ----- |
| `MCP_TOOL_TIMEOUT` padrão é de cerca de 28 horas quando não definido | [3] |
| Timeout por servidor (`"timeout"` no `.mcp.json`) é um limite de relógio; notificações de progresso não o estendem | [3] |
| Timeout de inatividade `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`: 30 min para stdio (stdio era isento antes da 2.1.203) | [3] |
| Notificações de progresso zeram o timer de inatividade | [3] |

**Consequências no papo:** `wait` espera no máximo 20 min (`MAX_WAIT_SECS = 1200`), abaixo da janela
de 30 min, e manda `notifications/progress` a cada 15 s quando o cliente passa um `progressToken`.

## 2. iroh 1.3: endpoints, descoberta e relays

| Ponto | Evidência |
| ----- | --------- |
| `Endpoint::builder(presets::N0)` liga o publicador e o resolvedor pkarr da n0, a busca por DNS e os relays padrão | [C] `iroh-1.3.0/src/endpoint/presets.rs` |
| Relays padrão: `use1-1`, `usw1-1`, `euc1-1` e `aps1-1` em `relay.n0.iroh.link` | [C] `iroh-1.3.0/src/defaults.rs` |
| Os quatro relays e `dns.iroh.link` respondem por HTTPS a partir da máquina de desenvolvimento | [M] `curl`, 2026-10-02 |
| Discar só com o endpoint id funciona: o endereço é resolvido por DNS/pkarr | [M] e2e público |
| O endpoint escolhe um relay "de casa" cerca de 0,5 s depois de subir, e publica o endereço com o relay cerca de 1 s depois de subir | [M] logs com `PAPO_LOG=iroh=info,iroh::address_lookup=debug` |
| Discar um endpoint que ainda não publicou o endereço falha com "No addressing information available", uns 3 s depois de começar | [M] mesmos logs |

## 3. iroh-gossip 0.101

| Ponto | Evidência |
| ----- | --------- |
| `Gossip::subscribe(topic, bootstrap)` volta na hora; `subscribe_and_join` espera um vizinho | [C] `iroh-gossip-0.101.0/src/api.rs` |
| Eventos: `NeighborUp`, `NeighborDown`, `Received`, `Lagged` | [C] `src/api.rs` |
| Tamanho máximo padrão de mensagem: 4096 bytes, ajustável com `Gossip::builder().max_message_size` | [C] `src/proto.rs`, `src/net.rs` |
| `Received` traz `delivered_from`, que é o vizinho que entregou, não o autor | [C] `src/api.rs` |
| **Rediscagem:** ao mandar para um par sem conexão, o ator marca o par como `Pending` e disca só se a fila dele estava vazia. Se a discagem falha, o par continua `Pending` com a fila cheia, e mensagens seguintes (inclusive de `join_peers`) só entram na fila: nunca há nova discagem | [C] `src/net.rs`, `handle_out_event` e `Dialer::queue_dial`; [M] logs mostram `SendMessage` sem `start to dial` depois da primeira falha |
| Uma conexão recebida para um par `Pending` vira `Active` e esvazia a fila | [C] `src/net.rs`, `handle_connection` |
| O protocolo de conexão é simétrico: streams unidirecionais nos dois sentidos | [C] `src/net/util.rs` |
| `Gossip::handle_connection(conn)` é público e trata a conexão como recebida | [C] `src/net.rs` |

**Consequências no papo:** a [ADR 0003](../adr/0003-o-papo-disca-os-pares-antes-do-gossip.md). O nó
assina o tópico sem bootstrap, disca cada membro com o ALPN do gossip, entrega a conexão a
`handle_connection` e só então chama `join_peers`.

## 4. Alternativas avaliadas

| Alternativa | O que se viu | Por que não |
| ----------- | ------------ | ----------- |
| Node/TypeScript com Hyperswarm e o SDK oficial de MCP | Um protótipo chegou a ser montado: `@modelcontextprotocol/sdk` 1.32 (revisão mais nova `2025-11-25`) aceita notificações com métodos desconhecidos; Hyperswarm faz hole punching por DHT | O mantenedor pediu um binário nativo para Linux, macOS e Windows; Node exige runtime |
| go-libp2p | Compila cruzado com facilidade | Travessia de NAT confiável exige relays de circuito e ajuste de DCUtR; relays públicos têm recursos limitados |
| Servidor WebSocket próprio | O mais simples e sempre alcançável | Alguém precisa hospedar; deixa de ser P2P |
| SDK de MCP em Rust | Existe | O papo precisa de capability experimental, notificação própria e long poll cancelável com progresso; à mão ficou menor ([ADR 0004](../adr/0004-o-servidor-mcp-e-escrito-a-mao.md)) |
| Conexões diretas sem gossip (malha própria) | Possível com iroh puro | O gossip já cuida de vizinhança e repasse em salas maiores; as garantias de entrega ficaram no `node.rs` |

## 5. Medições

Números de 2026-10-02. Como reproduzir cada um: [Desempenho](desempenho.md).

| Medida | Valor | Como |
| ------ | ----- | ---- |
| e2e público: convite, conexão, envio, push, resposta, `wait` | 6,04 s (3 execuções locais em momentos diferentes, depois do backoff) | [M] `cargo test --test mcp -- --ignored` |
| O mesmo teste com tentativas fixas a cada 10 s | 12,04 a 12,05 s (4 execuções) | [M] versão anterior do `node.rs` |
| O mesmo teste antes da discagem própria | nunca conectou em 90 s | [M] |
| Testes de integração do nó (5 testes, relay local) | cerca de 1,9 s | [M] `cargo test --test node` |
| Sobrecarga por frame | `msg` vazia: 203 bytes; `ack`: 80 bytes; `hello`: cerca de 200 bytes | [M] JSON do frame + 24 bytes de nonce + 16 de tag |
| Tamanho dos artefatos do CI (arquivo compactado + SHA-256) | 5,6 MB (macOS ARM64) a 7,1 MB (Linux x86_64 musl) | [M] run 37067969904 do workflow `release` |

## Fontes

1. Claude Code, Channels reference: https://code.claude.com/docs/en/channels-reference
2. Claude Code, Channels: https://code.claude.com/docs/en/channels
3. Claude Code, MCP (timeouts e push com channels): https://code.claude.com/docs/en/mcp
4. iroh 1.3.0 no crates.io: https://crates.io/crates/iroh (código lido localmente)
5. iroh-gossip 0.101.0 no crates.io: https://crates.io/crates/iroh-gossip (código lido localmente)

As páginas de channels foram resumidas por um agente de pesquisa e conferidas contra os pontos que o
papo usa; vale reler a documentação a cada versão do Claude Code, porque o recurso está em preview.
