# Arquitetura

O papo é um crate Rust com uma biblioteca (`src/lib.rs`) e um binário (`src/main.rs`). Os módulos
têm responsabilidades estreitas e dependem uns dos outros numa ordem só:

```text
room ──► proto ──► store ──► node ──► mcp ──► main (CLI)
                              ▲
                     net ─────┘ (endpoint na rede real; os testes usam uma rede local)
```

| Módulo | Responsabilidade |
| ------ | ---------------- |
| `room` | Segredo da sala, codificação do convite, derivação de tópico/chave/id, selagem dos frames. |
| `proto` | Frames do protocolo (`hello`, `msg`, `ack`), limites de tamanho, validação de nomes, ids. |
| `store` | Pasta do perfil: identidade, membros conhecidos, inbox, outbox, log e trava. |
| `net` | Cria o endpoint iroh na infraestrutura pública (ou com `PAPO_RELAY`). |
| `node` | O membro da sala: gossip, reconexão, entrega com confirmação, presença. |
| `mcp` | Servidor MCP via stdio: ferramentas, push por channels, `wait` com long poll. |
| `main` | Comandos da CLI (texto para pessoas em português). |

## O nó (`node`)

`Node::spawn` recebe um endpoint iroh já criado. Quem chama decide como ele chega à rede: o binário usa
a infraestrutura pública da n0; os testes usam um relay local e uma tabela de endereços em memória.
Isso deixa o nó inteiro testável sem internet.

Dentro do nó rodam duas tarefas:

- **Laço de eventos**: consome o fluxo do gossip (vizinho entrou, vizinho saiu, mensagem recebida) e
  trata cada frame.
- **Laço de manutenção**: a cada segundo, verifica se o nó está sozinho. Se estiver, disca os membros
  conhecidos com intervalo crescente (1, 2, 4, 8, 10, 10... segundos). Se estiver conectado, manda um
  `hello` e difunde a fila a cada 30 segundos.

O estado (membros, vizinhos, inbox, outbox, ids vistos) fica num `Mutex` que nunca é segurado durante
um `await`. Cada mudança no inbox ou na fila é gravada no disco na hora.

### Reconexão: o papo disca antes do gossip

Os membros não são passados ao gossip como pontos de entrada. O próprio papo disca cada membro com o
ALPN do gossip, entrega a conexão pronta ao gossip como se tivesse sido recebida, e só então pede ao
gossip para entrar em contato com aquele membro.

O motivo é um comportamento do iroh-gossip 0.101: se a primeira discagem para um ponto de entrada
falha, por exemplo porque o colega abriu o Claude um segundo antes e ainda não publicou o endereço,
aquele par fica "pendente" para sempre e novas tentativas não discam de novo. Uma conexão recebida
destrava esse estado, e o protocolo do gossip é simétrico. Detalhes na
[ADR 0003](adr/0003-o-papo-disca-os-pares-antes-do-gossip.md).

### Entrega

```text
 send()                                   receptor
   │ grava na outbox                        │
   │ difunde msg ───────────────────────────► descarta se não é para ele / é duplicata (re-ack)
   │                                        │ grava no inbox e no log
   │ ◄─────────────────────────────── ack ───┤ difunde ack
   │ tira da outbox, registra "delivered"   │ notifica o servidor MCP (push)
```

Se o `ack` não chega a tempo, `send` devolve "na fila" sem erro. A fila é difundida de novo quando um
vizinho se conecta e periodicamente. A [ADR 0005](adr/0005-entrega-pelo-menos-uma-vez-com-ack-e-fila.md)
explica as escolhas.

### Leitura e "marcar como lido"

- `wait` e `inbox` tiram as mensagens do inbox (e gravam o inbox no disco).
- Responder com `reply_to` tira do inbox a mensagem respondida e as anteriores do mesmo remetente,
  porque responder implica ter visto essas mensagens.
- O push por channels **não** marca nada como lido.

## O servidor MCP (`mcp`)

O servidor é escrito à mão sobre JSON-RPC, sem SDK
([ADR 0004](adr/0004-o-servidor-mcp-e-escrito-a-mao.md)). A estrutura:

- **Leitor**: lê o stdin linha a linha e despacha. `initialize`, `ping` e `tools/list` respondem na
  hora. Cada `tools/call` vira uma tarefa própria, guardada num mapa para poder ser cancelada por
  `notifications/cancelled`.
- **Escritor**: uma única tarefa escreve no stdout, a partir de um canal. Assim respostas e
  notificações nunca se misturam numa linha.
- **Inicialização em segundo plano**: o nó (trava do perfil, endpoint, gossip) sobe numa tarefa
  separada. O `initialize` é respondido imediatamente, e as ferramentas esperam o nó ficar pronto. Se a
  inicialização falha, as ferramentas devolvem o erro explicando o que fazer.
- **Bomba do channel**: depois de `notifications/initialized` e do nó pronto, assina os eventos do nó,
  empurra as mensagens não lidas que já estavam no inbox e passa a empurrar cada mensagem nova como
  `notifications/claude/channel`. Um conjunto de ids já empurrados evita repetições.

Quando o stdin fecha (o Claude Code encerrou a sessão), o servidor cancela as chamadas em andamento,
fecha o nó (o que avisa os vizinhos) e sai.

## A CLI (`main`)

Os comandos de configuração (`new`, `join`, `invite`, `install`) só mexem em arquivos. Os comandos que
falam com a sala (`say`, `status`) sobem um **nó descartável**: identidade nova a cada execução, sem
gravar inbox ou fila, sem confirmar mensagens e sem ser lembrado pelos outros. Isso permite usá-los ao
mesmo tempo que o servidor MCP do mesmo perfil, que é dono da identidade de verdade.

`log -f` lê o `log.jsonl` periodicamente em vez de usar notificações do sistema de arquivos, o que
funciona igual em Linux, macOS e Windows.

## Testes

- **Unitários**, nos módulos: convite, selagem, frames, nomes, negociação de versão, chaves do `meta`.
- **`tests/node.rs`**: nós de verdade (endpoints iroh e gossip reais) numa rede local com relay em
  processo. Cobrem entrega com ack, fila que sobrevive a reinício, mensagens com `to`, marcação de lido
  por resposta e a CLI descartável.
- **`tests/mcp.rs`**: dirige o binário compilado pelo stdio, como o Claude Code faz. Inclui um teste
  ignorado por padrão que põe dois servidores MCP para conversar pela internet real.
