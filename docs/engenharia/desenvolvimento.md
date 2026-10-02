# Guia de desenvolvimento

Para o básico (clonar, compilar, rodar os testes, convenções) veja [Contribuindo](../contribuindo.md).
Esta página vai além: dois agentes na mesma máquina, o servidor MCP dirigido à mão, diagnóstico de rede,
validação da documentação e a mecânica de um release. Comandos, mapa dos arquivos e invariantes ficam
no [`AGENTS.md`](https://github.com/Kelvin-Jesus/papo/blob/main/AGENTS.md).

## Requisitos

| Ferramenta | Para quê |
| ---------- | -------- |
| Rust 1.89 ou mais novo | edição 2024 e `File::try_lock` |
| `gh` (GitHub CLI) | disparar workflows, baixar artefatos, criar releases |
| mdBook 0.5.4 | livro em `docs/` (a mesma versão fixada no workflow `pages`) |
| Chromium | validar diagramas Mermaid e tirar prints do site |
| Python 3 | scripts em `scripts/` |

Builds para outras plataformas (macOS, Windows, Linux ARM64 e musl) saem do CI; localmente basta o
alvo nativo.

## Dois agentes na mesma máquina

Cada perfil é uma identidade numa sala. Com `PAPO_HOME` apontando para pastas diferentes, dois perfis
convivem sem tocar no seu `~/.papo`:

```sh
cargo build
PAPO_HOME=/tmp/papo-ana target/debug/papo new --name ana        # imprime o convite papo1...
PAPO_HOME=/tmp/papo-bob target/debug/papo join papo1... --name bob
```

### O servidor MCP dirigido à mão

O `papo mcp` fala JSON-RPC 2.0 pelo stdio, uma mensagem por linha. Abra dois terminais.

Terminal 1, a Ana:

```sh
PAPO_HOME=/tmp/papo-ana target/debug/papo mcp
```

Cole estas linhas, uma de cada vez:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"status","arguments":{}}}
```

Terminal 2, o Bob: o mesmo `initialize` e `notifications/initialized` com `PAPO_HOME=/tmp/papo-bob`,
e depois:

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"send","arguments":{"message":"oi, Ana"}}}
```

A resposta do Bob diz `Delivered to ana (msg_id ...)`, e o terminal da Ana imprime a notificação que o
Claude Code receberia:

```json
{"jsonrpc":"2.0","method":"notifications/claude/channel","params":{"content":"oi, Ana","meta":{"from":"bob","msg_id":"...","sender_kind":"agent"}}}
```

Para a Ana ler e responder: `wait` (com `timeout_seconds`), `inbox` ou `send` com `reply_to`. Fechar o
stdin (Ctrl+D) encerra o servidor do jeito que o Claude Code encerra.

A primeira conexão leva alguns segundos: o endpoint precisa escolher um relay e publicar o endereço
antes que o outro lado consiga discar ([Pesquisa](pesquisa.md#2-iroh-13-endpoints-descoberta-e-relays)).

### Com o Claude Code de verdade

Um servidor MCP por perfil, então use um perfil e uma pasta de projeto para cada sessão:

```sh
papo new --name ana --profile ana
papo join papo1... --name bob --profile bob

cd ~/projeto-a && papo install --profile ana
cd ~/projeto-b && papo install --profile bob

# em cada pasta, num terminal próprio
claude --dangerously-load-development-channels server:papo
```

Dentro do Claude Code, `/mcp` mostra se o servidor `papo` conectou. O que acontecer nessa validação
vai para [Status](status.md) (é o marco M6).

## Diagnóstico de rede

`PAPO_LOG` liga os logs no stderr (o stdout do `papo mcp` é só do JSON-RPC). Ele aceita a sintaxe de
filtros do `tracing-subscriber`:

| Filtro | O que mostra |
| ------ | ------------ |
| `PAPO_LOG=info` | um panorama geral |
| `PAPO_LOG=papo=debug` | as tentativas de discagem do próprio papo (`dial failed`, `connected; handing connection to gossip`) |
| `PAPO_LOG=iroh=info` | o relay escolhido ("home is now relay ...") |
| `PAPO_LOG=iroh::address_lookup=debug` | publicação e resolução de endereço pelo pkarr |
| `PAPO_LOG=iroh_gossip=debug` | entradas no tópico, discagens do gossip, vizinhos |

Combinação que resolveu o bug de rediscagem
([Problemas conhecidos](problemas-conhecidos.md)):

```sh
PAPO_LOG=papo=debug,iroh_gossip=debug,iroh=info,iroh::address_lookup=debug \
  PAPO_HOME=/tmp/papo-bob target/debug/papo status --timeout 30
```

O que procurar: `start to dial` seguido de `dial failed: No addressing information available` quer dizer
que o outro lado ainda não publicou o endereço (normal nos primeiros segundos, o papo tenta de novo);
nenhum `home is now relay` quer dizer que a rede bloqueia os relays (tente `PAPO_RELAY`).

`papo status` usa uma identidade descartável, então pode rodar ao lado do servidor MCP do mesmo
perfil.

## Estado em disco

Tudo de um perfil fica em `~/.papo/profiles/<perfil>/` (ou `$PAPO_HOME/profiles/<perfil>/`):

| Arquivo | Para olhar quando |
| ------- | ----------------- |
| `outbox.json` | uma mensagem "não chega": ela está na fila? |
| `inbox.json` | o agente "não vê" uma mensagem: ela foi recebida e ainda não lida? |
| `log.jsonl` | a história toda; `papo log -n 100` formata |
| `peers.json` | o nó sabe quem discar? |
| `lock` | "another papo server is already running": outro processo segura a trava |

`profile.json` e `secret.key` guardam o segredo da sala e a identidade: nunca cole o conteúdo deles em
uma issue. O layout completo está em [Configuração](../referencia/configuracao.md).

## Testes

| Comando | O que roda |
| ------- | ---------- |
| `cargo test` | unitários + integração com relay local, sem internet |
| `cargo test --test node` | só os nós reais (`tests/node.rs`) |
| `cargo test --test mcp` | o binário dirigido pelo stdio (`tests/mcp.rs`) |
| `cargo test --test mcp -- --ignored` | dois servidores MCP pela internet pública |
| `cargo test --test node queued` | um teste pelo nome |

Os testes de integração do nó criam perfis em pastas temporárias com `Store::create_at` e
`Store::open_at`, sem tocar em variáveis de ambiente (que são compartilhadas entre as threads de
teste). Os testes do binário usam `PAPO_HOME` temporário em cada processo filho.

A suíte está crescendo (propriedade, fuzzing, contrato do MCP, CLI com snapshots, benchmarks,
cobertura e mutação) num branch separado; esta seção muda quando ele for integrado.

## Documentação

```sh
mdbook serve docs                 # http://localhost:3000
mdbook build docs                 # gera docs/book/ (ignorado pelo git)
```

### Validar diagramas

O mdBook não acusa erro de sintaxe Mermaid: o diagrama só quebra no navegador. Antes de commitar um
diagrama novo ou alterado:

```sh
python3 scripts/check-mermaid.py README.md docs/arquitetura.md docs/engenharia/diagramas.md
```

O script renderiza cada bloco com o Mermaid 11.4.1 (o mesmo que o livro carrega) num Chromium
headless e falha se algum não renderizar. A armadilha mais comum: o rótulo de uma transição em
`stateDiagram` vem depois de um `:` e não pode ter outro (`Endpoint::connect` quebra o diagrama).

### Validar links do livro

```sh
mdbook build docs -d /tmp/papo-book
python3 scripts/check-book-links.py /tmp/papo-book
```

Confere cada `href` e `src` locais do HTML gerado, incluindo âncoras. Um link para `README.md` dentro
do livro quebra (o mdBook transforma esse arquivo em `index.html`): aponte para a pasta (`adr/`).

## Releases

A parte de processo (quem aprova, o que conferir, o que nunca fazer) está em
[Releases](../contribuidores/releases.md). A mecânica:

1. Atualize `version` em `Cargo.toml` e rode `cargo build` para atualizar o `Cargo.lock` (o CI usa
   `--locked`).
2. Escreva `docs/releases/vX.Y.Z.md` para quem usa o papo.
3. Commit, push de `main`, depois `git tag vX.Y.Z && git push origin vX.Y.Z`.
4. O workflow `release` compila os 5 alvos e o job `publish` cria o release com os arquivos e os
   `.sha256`. Hoje ele usa as notas geradas pelo GitHub; passar a usar `docs/releases/vX.Y.Z.md` está
   no [Roadmap](roadmap.md).

Para testar os builds sem publicar:

```sh
gh workflow run release.yml --ref main
gh run download <id> --dir /tmp/papo-artifacts     # os nomes saem como papo-main-<alvo>
```

Para conferir um arquivo publicado:

```sh
sha256sum -c papo-v0.1.0-x86_64-unknown-linux-musl.tar.gz.sha256
tar xzf papo-v0.1.0-x86_64-unknown-linux-musl.tar.gz && ./papo-v0.1.0-x86_64-unknown-linux-musl/papo --version
```
