# Docker

O papo também roda em container. Serve para quem não quer instalar o binário, para ligar o
servidor MCP no Claude Code a partir de uma imagem, para subir um relay próprio e para rodar os
testes num Linux fixo.

## A imagem

`ghcr.io/kelvin-jesus/papo`, publicada a cada tag `vX.Y.Z` para `linux/amd64` e `linux/arm64`
(tags `X.Y.Z`, `X.Y` e `latest`). Enquanto não houver uma tag publicada, construa a sua:

```sh
docker build -t papo .
```

Nos exemplos abaixo, troque `ghcr.io/kelvin-jesus/papo` por `papo` se você construiu localmente.

| Característica | Valor |
| -------------- | ----- |
| Binário | estático (musl), o mesmo `papo` dos releases |
| Base | `gcr.io/distroless/static-debian12:nonroot`: sem shell, com certificados e fusos horários |
| Usuário | não-root (UID 65532) |
| Tamanho | cerca de 8 MB para baixar (comprimida), cerca de 19 MB descompactada |
| Dados | `PAPO_HOME=/data`, um volume: perfil, identidade, inbox, outbox e log |

Os certificados não são enfeite: o papo publica e consulta endereços no DNS da n0 por HTTPS, e
essa verificação usa o repositório de certificados do sistema.

## Usar a CLI

Guarde o perfil num volume com nome, para ele sobreviver entre execuções:

```sh
docker run --rm -v papo-data:/data ghcr.io/kelvin-jesus/papo new --name kj --about api-pagamentos
docker run --rm -v papo-data:/data ghcr.io/kelvin-jesus/papo invite
docker run --rm -v papo-data:/data ghcr.io/kelvin-jesus/papo status
docker run --rm -v papo-data:/data -e TZ=America/Sao_Paulo ghcr.io/kelvin-jesus/papo log -n 20
```

Um alias deixa o uso igual ao do binário:

```sh
alias papo='docker run --rm -i -v papo-data:/data -e TZ=America/Sao_Paulo ghcr.io/kelvin-jesus/papo'
papo say --to ana "Pode olhar o PR do webhook?"
```

`TZ` só muda o horário mostrado em `papo log`; sem ele, o container mostra UTC.

## O servidor MCP para o Claude Code

O Claude Code fala com o servidor por stdio, então o container precisa de `-i` e **não** pode ter
`-t` (um TTY mistura caracteres de controle no JSON-RPC):

```sh
claude mcp add papo -- docker run -i --rm -v papo-data:/data ghcr.io/kelvin-jesus/papo mcp
claude --dangerously-load-development-channels server:papo
```

A flag de *channels* continua sendo do lado do Claude Code, igual ao uso sem Docker.

Cuidados:

- **Um container por perfil.** O servidor trava o perfil para que duas sessões não disputem a
  mesma identidade na rede. Uma segunda sessão do Claude Code com o mesmo volume recebe "another
  papo server is already running with this profile". Para estar em duas salas, use dois volumes ou
  `--profile`.
- **`--about` no `new`/`join`.** Fora do Docker, o papo usa o nome da pasta do projeto para dizer
  aos colegas no que você está trabalhando. No container a pasta de trabalho é `/`, que não tem
  nome, então informe `--about` ao criar ou entrar na sala.
- **`papo install` não serve aqui.** Rodado dentro do container, ele imprime o caminho interno
  (`/usr/local/bin/papo`), que não existe no seu computador. Use o `claude mcp add` acima.
- **`say` e `log` ao mesmo tempo que o servidor.** Funcionam num segundo container com o mesmo
  volume: eles não pegam a trava do perfil.
- **Rede.** O container fica atrás do NAT do Docker e o papo atravessa como em qualquer NAT (hole
  punching, ou relay quando não dá). No Linux, `--network host` aumenta a chance de conexão direta.
- **`PAPO_RELAY` vazia quebra.** Se for usar relay próprio, passe a URL completa
  (`-e PAPO_RELAY=http://...`); não defina a variável vazia.

## Relay próprio com Docker Compose

Para quem não pode (ou não quer) usar os relays públicos da n0, há um `iroh-relay` pronto, na mesma
versão que o papo usa:

```sh
docker compose -f docker/relay/compose.yaml up -d      # escuta em http://<este-host>:3340
```

Em **todos** os membros da sala:

```sh
export PAPO_RELAY=http://<este-host>:3340
# ou, com Docker: docker run ... -e PAPO_RELAY=http://<este-host>:3340 ...
```

> Este relay roda em modo de desenvolvimento: HTTP sem TLS e sem controle de acesso. Serve para
> rede local, testes e demonstrações. As mensagens continuam cifradas de ponta a ponta com a chave
> da sala, mas qualquer um que alcance a porta pode usar o relay. Para produção, rode o
> `iroh-relay` com TLS (veja o [repositório do iroh](https://github.com/n0-computer/iroh/tree/main/iroh-relay))
> e aponte `PAPO_RELAY` para a URL `https://`.

O relay próprio substitui os relays públicos para o tráfego, mas os membros ainda se encontram pelo
DNS da n0 (é ali que cada um publica o endereço atual), então a internet continua necessária. Veja
também [Relay próprio](relay-proprio.md).

## Testes em container

| Comando | O que faz | Precisa de internet |
| ------- | --------- | ------------------- |
| `docker build --target test .` | Roda `cargo test --locked` num Linux fixo (Alpine, musl): os mesmos testes herméticos do CI | Só para baixar dependências |
| `scripts/docker-smoke.sh [imagem]` | CLI, volume nomeado, servidor MCP por stdio (`initialize`, `tools/list`, `status`) e a trava de perfil | Não |
| `scripts/docker-e2e.sh` | Dois agentes (ana e bob) em containers separados, com um relay local: bob pergunta, ana recebe por push (`claude/channel`), responde com `reply_to`, bob recebe com `wait`, e o humano do bob fala com `papo say` | Sim (DNS da n0) |
| `scripts/docker-e2e.sh --public` | O mesmo, pelos relays públicos da n0 | Sim |

O e2e dirige os dois servidores MCP pelo stdio, do mesmo jeito que duas sessões do Claude Code
fariam, e sai com erro se qualquer etapa falhar. Com `PAPO_LOG=iroh::socket::transports::relay=info`
ele mostra no fim qual relay cada agente usou. `PAPO_E2E_SKIP_BUILD=1` reaproveita imagens já
construídas.

No GitHub, o workflow `docker` constrói a imagem e roda o smoke test em cada PR e push na `main`
que mexa no código ou no Docker, roda o e2e como informativo e, nas tags `v*`, publica a imagem
multi-arquitetura no GHCR.

## Multi-arquitetura

```sh
docker buildx build --platform linux/amd64,linux/arm64 -t papo .
```

O estágio de compilação roda sempre na arquitetura da sua máquina e compila de forma cruzada com
zig quando o alvo é outro, então nada de Rust roda emulado. O primeiro build ARM64 leva alguns
minutos a mais para instalar o `cargo-zigbuild`.
