# Contribuindo

Contribuições são bem-vindas: correções, testes, documentação e ideias do
[roadmap](https://github.com/Kelvin-Jesus/papo/wiki/Roadmap). Para mudanças maiores, abra uma
[issue](https://github.com/Kelvin-Jesus/papo/issues) antes para discutir a abordagem.

## Preparando o ambiente

Requisitos: Rust 1.89 ou mais novo (o projeto usa `File::try_lock` e a edição 2024).

```sh
git clone https://github.com/Kelvin-Jesus/papo
cd papo
cargo build
```

## Ciclo de desenvolvimento

```sh
cargo build
cargo test                                   # unitários + integração (rede local, sem internet)
cargo test --test mcp -- --ignored           # dois servidores MCP pela internet real
cargo clippy --all-targets -- -D warnings
cargo fmt                                    # rustfmt.toml: max_width 120
```

Para ver o que acontece na rede durante um teste manual:

```sh
PAPO_LOG=iroh_gossip=debug,iroh=info target/debug/papo status
```

Para testar dois perfis na mesma máquina sem mexer no seu `~/.papo`, use `PAPO_HOME`:

```sh
PAPO_HOME=/tmp/a target/debug/papo new --name ana
PAPO_HOME=/tmp/b target/debug/papo join <convite> --name bob
```

## Testes

- Testes de integração usam nós reais (endpoints iroh e gossip) numa rede local com relay em processo.
  Prefira esse estilo a mocks.
- Qualquer teste que dependa da internet deve ser `#[ignore]`.
- `tests/mcp.rs` dirige o binário pelo stdio. Ao adicionar uma ferramenta MCP, cubra-a ali.

## Convenções

- Código, comentários e textos lidos pelo agente (instruções do servidor, descrições e respostas das
  ferramentas) em inglês. Saída da CLI, README e documentação em português.
- Comentários explicam o porquê, não o quê.
- Erros sobem com contexto (`anyhow::Context`); falhas de gravação em segundo plano vão para o stderr.
- O stdout do `papo mcp` é exclusivo do JSON-RPC.

As invariantes que não podem ser quebradas (e o mapa dos arquivos) estão no
[`AGENTS.md`](https://github.com/Kelvin-Jesus/papo/blob/main/AGENTS.md). Decisões de arquitetura
ficam em [ADRs](adr/); se a sua mudança contraria uma delas, escreva uma ADR nova.

## Documentação

Este livro é feito com [mdBook](https://rust-lang.github.io/mdBook/) a partir da pasta `docs/`:

```sh
mdbook serve docs        # http://localhost:3000, recarrega ao salvar
mdbook build docs        # gera docs/book/
```

O site do projeto (página inicial + este livro) é publicado no GitHub Pages pelo workflow `pages`.

## Releases

1. Atualize a versão em `Cargo.toml` e rode `cargo build` para atualizar o `Cargo.lock`.
2. Faça commit e crie a tag: `git tag v0.2.0 && git push origin v0.2.0`.
3. O workflow `release` compila para Linux (musl, x86_64 e ARM64), macOS (Intel e Apple Silicon) e
   Windows, e publica os arquivos com SHA-256 na página de Releases.

O workflow também pode ser disparado à mão (`workflow_dispatch`) para testar os builds sem publicar.
