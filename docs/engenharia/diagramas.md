# Diagramas

Os diagramas Mermaid das partes que se mexem, reunidos numa página. O texto de referência continua
sendo a [Arquitetura](../arquitetura.md), o [Protocolo](../referencia/protocolo.md) e as
[ADRs](../adr/); quando um diagrama mudar, atualize o texto junto. Todos os blocos desta página são
renderizados num Chromium headless antes de cada commit que os altera
([Desenvolvimento](desenvolvimento.md#validar-diagramas)).

## Implantação

Quem fala com quem quando duas pessoas usam o papo. Nada roda num servidor do projeto: os únicos
serviços externos são a descoberta de endereço e os relays da n0.

```mermaid
flowchart LR
  subgraph A["Máquina de quem convida"]
    CCA["Claude Code"] <-->|"MCP via stdio"| PA["papo mcp"]
    PA --- DA[("~/.papo/profiles/default")]
    HA(["humano: papo log -f, papo say"]) -.-> DA
  end
  subgraph B["Máquina de quem entra"]
    PB["papo mcp"] <-->|"MCP via stdio"| CCB["Claude Code"]
    PB --- DB[("~/.papo/profiles/default")]
  end
  subgraph N0["Infraestrutura pública da n0"]
    DNS[("DNS e pkarr<br/>dns.iroh.link")]
    REL(["relays<br/>*.relay.n0.iroh.link"])
  end
  PA <==>|"QUIC direto, hole punching"| PB
  PA -.->|"publica e resolve endereço"| DNS
  PB -.->|"publica e resolve endereço"| DNS
  PA -.->|"reserva cifrada"| REL
  REL -.-> PB
```

## Componentes

Os módulos do crate e quem usa cada um. Os testes trocam a rede pública por um relay em processo.

```mermaid
flowchart TD
  main["main.rs<br/>CLI"] --> mcp["mcp.rs<br/>servidor MCP"]
  main --> node
  main --> net["net.rs<br/>endpoint público"]
  main --> store
  mcp --> node["node.rs<br/>membro da sala"]
  mcp --> store
  node --> store["store.rs<br/>perfil em disco"]
  node --> proto["proto.rs<br/>frames"]
  node --> room["room.rs<br/>segredo, convite, cifra"]
  proto --> room
  store --> proto
  tnode(["tests/node.rs<br/>relay local"]) -.-> node
  tmcp(["tests/mcp.rs<br/>binário real"]) -.-> main
```

## Uma conversa

O caso de referência: dois agentes combinando um contrato sem humano no meio.

```mermaid
sequenceDiagram
  actor V as Você
  participant CV as Seu Claude
  participant PV as Seu papo
  participant PC as papo do colega
  participant CC as Claude do colega
  actor C as Colega
  V->>CV: combina o webhook com o agente do colega
  CV->>PV: tools/call send
  PV->>PC: msg selada com a chave da sala
  PC-->>PV: ack
  PV-->>CV: Delivered to colega
  PC->>CC: notifications/claude/channel
  CC->>CC: lê o código do colega
  CC->>PC: tools/call send com reply_to
  PC->>PV: msg
  PV-->>PC: ack
  PV->>CV: notifications/claude/channel
  CV-->>V: contrato combinado
  CC-->>C: o que ficou combinado
```

## Entrega com fila offline

O caminho de uma mensagem quando o destinatário está fora do ar
([ADR 0005](../adr/0005-entrega-pelo-menos-uma-vez-com-ack-e-fila.md)).

```mermaid
sequenceDiagram
  participant S as papo de quem envia
  participant D as disco de quem envia
  participant R as papo de quem recebe
  participant DR as disco de quem recebe
  S->>D: grava na outbox
  S->>S: difunde msg (ninguém ouve)
  S-->>S: sem ack em 8 s, devolve "queued"
  Note over S,R: o destinatário volta
  R->>S: conexão e hello
  S->>R: reenvia toda a outbox
  R->>DR: grava inbox e log
  R-->>S: ack
  S->>D: tira da outbox, log "delivered"
  Note over S,R: se o ack se perder, a mensagem fica na outbox e volta a cada 30 s, e o destinatário só confirma de novo
```

## Reconexão

O laço de manutenção do nó, que existe por causa do comportamento de rediscagem do iroh-gossip
([ADR 0003](../adr/0003-o-papo-disca-os-pares-antes-do-gossip.md)).

```mermaid
stateDiagram-v2
  [*] --> Sozinho: nó sobe, tópico sem bootstrap
  Sozinho --> Discando: chegou a hora da tentativa
  Discando --> Sozinho: falhou ou passou de 15 s
  Discando --> Conectado: conexão entregue ao gossip, join_peers
  Conectado --> Conectado: hello e reenvio da outbox a cada 30 s
  Conectado --> Sozinho: último vizinho saiu
  note right of Sozinho
    espera 1, 2, 4, 8, 10, 10 s
    e volta a 1 s ao conectar
  end note
```

## Servidor MCP por dentro

Uma tarefa por chamada de ferramenta, um único escritor no stdout, e a bomba do channel que só começa
depois do `notifications/initialized`
([ADR 0004](../adr/0004-o-servidor-mcp-e-escrito-a-mao.md)).

```mermaid
flowchart LR
  IN(["stdin"]) --> L["leitor"]
  L -->|"initialize, ping, tools/list"| W["escritor único"]
  L -->|"tools/call"| T["tarefa por chamada"]
  L -->|"notifications/cancelled"| C["aborta a tarefa"]
  C -.-> T
  T --> W
  T <--> N["nó"]
  S["subida em segundo plano<br/>trava, endpoint, gossip"] --> N
  N -->|"mensagem nova"| B["bomba do channel"]
  B -->|"notifications/claude/channel"| W
  T -->|"notifications/progress (wait)"| W
  W --> OUT(["stdout: só JSON-RPC"])
```

## Subida do `papo mcp`

O `initialize` é respondido na hora; as ferramentas esperam o nó ficar pronto.

```mermaid
sequenceDiagram
  participant CC as Claude Code
  participant M as papo mcp
  participant BG as tarefa de subida
  participant G as gossip
  CC->>M: initialize
  M-->>CC: capabilities, instructions
  M->>BG: inicia em paralelo
  BG->>BG: abre o perfil e pega a trava
  BG->>G: endpoint N0 e assinatura do tópico
  CC->>M: notifications/initialized
  CC->>M: tools/call status
  Note over M: a chamada espera o nó
  BG-->>M: nó pronto
  M-->>CC: resultado do status
  M->>CC: mensagens não lidas viram notificações de channel
```

## Um frame

Do envelope aos bytes na rede ([Protocolo](../referencia/protocolo.md)).

```mermaid
flowchart LR
  E["Envelope<br/>id, from, node, kind, to, reply_to, ts, body"] --> F["Frame msg<br/>JSON com t igual a msg"]
  F --> K["selagem XChaCha20-Poly1305<br/>chave: derive_key papo v1 frame key<br/>AAD: papo/v1"]
  K --> X["nonce de 24 bytes + texto cifrado + tag de 16"]
  X --> G["broadcast no tópico<br/>derive_key papo v1 gossip topic"]
```

## Perfil em disco

Quem escreve e quem lê cada arquivo de `~/.papo/profiles/<perfil>/`
([Configuração](../referencia/configuracao.md)).

```mermaid
flowchart LR
  subgraph P["~/.papo/profiles/perfil"]
    prof[("profile.json<br/>nome, segredo da sala")]
    key[("secret.key<br/>identidade")]
    peers[("peers.json<br/>membros conhecidos")]
    inbox[("inbox.json")]
    outbox[("outbox.json")]
    log[("log.jsonl")]
    lock[("lock")]
  end
  NEWJOIN["papo new / join"] --> prof
  NEWJOIN --> key
  NEWJOIN --> peers
  MCP["papo mcp"] --> lock
  MCP --> inbox
  MCP --> outbox
  MCP --> log
  MCP --> peers
  prof --> MCP
  key --> MCP
  LOGCMD["papo log"] -.-> log
  EPH["papo say / status<br/>identidade descartável"] -.-> peers
  EPH -.->|"status conta"| inbox
  EPH -.->|"status conta"| outbox
```

## CI, release, site e wiki

Os quatro workflows do GitHub Actions e o que dispara cada um.

```mermaid
flowchart TD
  PUSH(["push em main"]) --> CI["ci<br/>fmt, clippy, testes em Linux, macOS e Windows<br/>+ e2e na internet (informativo)"]
  PUSH -->|"mudou docs/, site/ ou assets/"| PAGES["pages<br/>mdBook + site em _site"]
  PUSH -->|"mudou wiki/"| WIKI["wiki<br/>espelha wiki/ na wiki do GitHub"]
  TAG(["tag vX.Y.Z"]) --> REL["release<br/>5 plataformas, SHA-256"]
  DISPATCH(["workflow_dispatch"]) -.-> REL
  DISPATCH -.-> PAGES
  DISPATCH -.-> WIKI
  PAGES --> GHP[["GitHub Pages<br/>kelvin-jesus.github.io/papo"]]
  REL --> GHR[["GitHub Releases<br/>só com tag"]]
  WIKI --> GHW[["wiki do GitHub"]]
```
