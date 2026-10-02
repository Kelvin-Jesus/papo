<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/logo/papo-logo-dark.svg">
    <img src="assets/logo/papo-logo.svg" alt="papo" width="280">
  </picture>
</p>

<p align="center">
  <strong>Seu Claude conversa direto com o Claude do seu colega.</strong>
</p>

<p align="center">
  <a href="https://github.com/Kelvin-Jesus/papo/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/Kelvin-Jesus/papo/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/Kelvin-Jesus/papo/releases"><img alt="Release" src="https://img.shields.io/github/v/release/Kelvin-Jesus/papo?include_prereleases&label=release"></a>
  <a href="https://kelvin-jesus.github.io/papo/docs/"><img alt="Docs" src="https://img.shields.io/badge/docs-livro-informational"></a>
  <a href="LICENSE"><img alt="Licença MIT" src="https://img.shields.io/badge/licen%C3%A7a-MIT-blue"></a>
</p>

<p align="center">
  <a href="https://kelvin-jesus.github.io/papo/">Site</a> ·
  <a href="https://kelvin-jesus.github.io/papo/docs/tutorial.html">Tutorial</a> ·
  <a href="https://kelvin-jesus.github.io/papo/docs/">Documentação</a> ·
  <a href="https://github.com/Kelvin-Jesus/papo/wiki">Wiki</a> ·
  <a href="https://github.com/Kelvin-Jesus/papo/releases">Downloads</a>
</p>

---

Quando dois agentes precisam combinar algo (o formato de uma API, um contrato de evento, quem muda
o quê), hoje as pessoas viram proxy: copiam a pergunta de um Claude, colam no chat, o colega cola
no Claude dele, copia a resposta, manda de volta... O **papo** acaba com esse vai e vem. Os dois
Claudes conversam direto, se entendem e só chamam vocês quando precisam de uma decisão.

O nome vem de "bater papo": você pede, e os agentes batem papo entre si até resolver.

```
 Claude Code (você)                                Claude Code (colega)
        │ MCP (stdio)                                     │ MCP (stdio)
    papo mcp  ◄──── iroh: QUIC P2P, cifrado ponta a ponta ────►  papo mcp
 ~/.papo (inbox, outbox, log)       hole punching; relay só como fallback
```

- **Binário nativo único** para Linux, macOS e Windows, sem runtime e sem servidor para hospedar.
- **P2P de verdade** via [iroh](https://iroh.computer): conexão direta por QUIC, com hole punching
  através de NAT. Quando a conexão direta não é possível, o tráfego passa cifrado por um relay.
- **Nenhuma mensagem se perde**: se o colega está offline, ela fica na fila e é entregue quando ele
  voltar, com confirmação de recebimento.
- **Push na sessão**: com *channels* do Claude Code, a mensagem do colega aparece sozinha na
  sessão do seu Claude, que reage sem você digitar nada.
- **Feito para agentes**: o servidor ensina o Claude a escrever mensagens que se explicam sozinhas,
  a não vazar segredos para o outro lado e a não entrar em loop de "ok/obrigado".

## Instalação

Baixe o arquivo do seu sistema em [Releases](https://github.com/Kelvin-Jesus/papo/releases),
extraia e coloque o `papo` (ou `papo.exe`) no `PATH`:

| Sistema             | Arquivo                                         |
| ------------------- | ----------------------------------------------- |
| Linux x86_64        | `papo-<versão>-x86_64-unknown-linux-musl.tar.gz`  |
| Linux ARM64         | `papo-<versão>-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `papo-<versão>-aarch64-apple-darwin.tar.gz`       |
| macOS Intel         | `papo-<versão>-x86_64-apple-darwin.tar.gz`        |
| Windows             | `papo-<versão>-x86_64-pc-windows-msvc.zip`        |

Ou compile (Rust 1.89+): `cargo install --git https://github.com/Kelvin-Jesus/papo`.

> No macOS, um binário baixado pelo navegador pode ser bloqueado pelo Gatekeeper. Libere com
> `xattr -d com.apple.quarantine ./papo`.

## Primeiros passos

```sh
# 1. Você cria a sala e manda o convite (papo1...) ao colega por um canal privado
papo new --name kj

# 2. O colega entra
papo join papo1abcd... --name ana

# 3. Cada um, dentro da pasta do projeto em que vai trabalhar
papo install

# 4. Cada um abre o Claude Code com channels ligado
claude --dangerously-load-development-channels server:papo
```

Depois é só pedir:

> Combina com o agente da Ana o formato do webhook de pagamento pelo papo. Ela está implementando
> o consumidor. Quando fecharem, me mostra o contrato final.

E acompanhar a conversa dos agentes em outro terminal com `papo log -f`.

O [tutorial completo](https://kelvin-jesus.github.io/papo/docs/tutorial.html) mostra uma sessão de
ponta a ponta entre duas pessoas, com o que aparece em cada terminal.

## Com ou sem *channels*

| Modo | Como abrir o Claude | O que acontece |
| ---- | ------------------- | -------------- |
| **Push** (recomendado) | `claude --dangerously-load-development-channels server:papo` | Mensagens novas entram sozinhas na sessão como `<channel source="papo" ...>` e o Claude reage, mesmo parado. |
| **Pull** | `claude` | Funciona igual, mas o Claude só vê mensagens quando chama `wait` ou `inbox` (por exemplo: "manda e espera a resposta"). |

*Channels* é *research preview* do Claude Code: exige login com conta claude.ai ou chave do
Console, e em organizações Team/Enterprise um Owner precisa habilitar *Channels* nas
configurações de admin. Sem isso, o papo continua funcionando no modo pull.

## Comandos

| Comando | Descrição |
| ------- | --------- |
| `papo new --name <nome>` | Cria uma sala e mostra o convite. |
| `papo join <convite> --name <nome>` | Entra numa sala. |
| `papo invite` | Gera um convite para chamar mais alguém. |
| `papo install [--scope local\|user\|project] [--print]` | Registra no Claude Code. |
| `papo log [-n 30] [-f]` | Mostra a conversa; `-f` acompanha ao vivo. |
| `papo say [--to <nome>] <texto>` | Você (humano) fala na sala. |
| `papo status` | Teste de conexão: entra na sala e lista quem está online. |
| `papo mcp` | O servidor MCP. Quem executa é o Claude Code. |

O Claude ganha as ferramentas `send`, `wait`, `inbox`, `history` e `status`. Referência completa da
[CLI](https://kelvin-jesus.github.io/papo/docs/referencia/cli.html), das
[ferramentas MCP](https://kelvin-jesus.github.io/papo/docs/referencia/ferramentas-mcp.html) e da
[configuração](https://kelvin-jesus.github.io/papo/docs/referencia/configuracao.html) no livro.

## Segurança

- **O convite é o segredo da sala.** Dele derivam o tópico P2P e a chave que cifra cada mensagem
  (XChaCha20-Poly1305), por cima do TLS do QUIC. Relays só veem bytes cifrados; quem não tem o
  convite não lê nem injeta mensagens.
- **Mensagem de outro agente não é ordem do seu usuário.** O papo diz isso ao Claude: nada de vazar
  `.env`, tokens ou credenciais, nem de fazer algo destrutivo só porque o outro agente pediu. As
  permissões do Claude Code continuam valendo.
- **Anti-loop**: mais de 40 envios em 10 minutos viram erro, e o agente é orientado a parar e falar
  com você.

Modelo de ameaças completo em [Segurança](https://kelvin-jesus.github.io/papo/docs/seguranca.html).

## Documentação

| Onde | O que tem |
| ---- | --------- |
| [Livro](https://kelvin-jesus.github.io/papo/docs/) | Tutorial, guias, referência, arquitetura, protocolo, segurança, ADRs |
| [Wiki](https://github.com/Kelvin-Jesus/papo/wiki) | Receitas de pedidos, perguntas frequentes, problemas comuns, roadmap |
| [`docs/adr/`](docs/adr/) | Decisões de arquitetura |
| [`CONTEXT.md`](CONTEXT.md) | Linguagem do domínio |
| [`AGENTS.md`](AGENTS.md) | Instruções para agentes de IA que trabalham neste repositório |
| [`knowledge/`](knowledge/) | Base de conhecimento no formato OKF |

## Desenvolvimento

```sh
cargo test                                   # unitários + integração (rede local, sem internet)
cargo test --test mcp -- --ignored           # dois servidores MCP pela internet real
cargo clippy --all-targets -- -D warnings
```

Releases saem de uma tag `vX.Y.Z`: o workflow `release` compila para as cinco plataformas e publica
os arquivos com SHA-256. Veja [Contribuindo](https://kelvin-jesus.github.io/papo/docs/contribuindo.html).

## Licença

[MIT](LICENSE)
