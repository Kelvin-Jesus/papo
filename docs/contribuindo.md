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
cargo test --features test-network          # todas as suítes, sem internet (veja Testes)
cargo test --test mcp -- --ignored           # dois servidores MCP pela internet real
cargo clippy --all-targets --features test-network -- -D warnings
cargo fmt                                    # rustfmt.toml: max_width 120
```

Para ver o que acontece na rede durante um teste manual:

```sh
PAPO_LOG=iroh_gossip=debug,iroh=info target/debug/papo status
```

Para testar dois perfis na mesma máquina sem mexer no seu `~/.papo`, use `PAPO_HOME`:

```sh
PAPO_HOME=/tmp/a target/debug/papo new --name voce
PAPO_HOME=/tmp/b target/debug/papo join <convite> --name colega
```

## Testes

`cargo test --features test-network` roda tudo o que não precisa de internet. Sem a feature, só o
ponta a ponta hermético fica de fora. Cada tipo de teste tem um lugar:

| Tipo | Onde | O que garante |
| ---- | ---- | ------------- |
| Unitários | `src/*.rs` (`mod tests`) | Funções puras: cifra, convite, frames, formatação, limitador, helpers da CLI |
| Doc tests | exemplos em `src/room.rs`, `src/proto.rs` | A API pública documentada funciona como mostrada |
| Propriedades | `tests/properties.rs` (proptest) | Invariantes para qualquer entrada: convite vai e volta, truncado é rejeitado, bit trocado é detectado |
| Robustez | `tests/robustness.rs` | Entrada arbitrária nunca causa pânico; o servidor MCP sobrevive a rajadas de linhas aleatórias |
| Estado em disco | `tests/store.rs` | Escrita atômica, arquivos corrompidos, trava do perfil, permissões |
| Nó | `tests/node.rs`, `tests/node_scenarios.rs` | Entrega, fila, ack, duplicatas, reinício, crash, ordem, concorrência, salas maiores |
| Contrato MCP | `tests/mcp.rs`, `tests/mcp_contract.rs` | JSON-RPC, esquemas das ferramentas, push, progresso, textos que o agente lê |
| CLI | `tests/cli.rs` | Comandos, mensagens de erro e o que fica no disco |
| Ponta a ponta | `tests/e2e_local.rs` (feature `test-network`) | Dois `papo mcp` reais e a CLI conversando por um relay local |
| Internet real | `cargo test --test mcp -- --ignored` | O mesmo, pela infraestrutura pública da n0 |

Regras:

- Testes de rede usam nós reais (endpoints iroh e gossip) numa rede local com relay em processo
  (`tests/common/localnet.rs`). Prefira esse estilo a mocks. Para mandar frames que um nó normal
  nunca mandaria, use o `RawPeer` do mesmo arquivo.
- Testes que dirigem o binário usam `tests/common/mod.rs`, sempre com um `PAPO_HOME` temporário.
- Qualquer teste que dependa da internet deve ser `#[ignore]`.
- Ao adicionar uma ferramenta MCP, cubra-a em `tests/mcp_contract.rs` e atualize o snapshot.

### Snapshots

`tests/snapshots/` guarda o `initialize`, o `tools/list`, o `--help` e a saída do `new`. Se você mudou
algo disso de propósito, regrave e revise o diff:

```sh
INSTA_UPDATE=always cargo test --features test-network
git diff tests/snapshots
```

Na CI os snapshots nunca são regravados: diferença é falha.

### Cobertura

```sh
cargo llvm-cov --features test-network --summary-only
```

A CI falha abaixo de 90% das linhas (medido em 94% quando o piso foi definido) e publica o resumo na
página da execução. Os processos `papo` que os testes iniciam também contam, porque terminam por EOF
no stdin em vez de serem mortos.

### Fuzzing

Os alvos em `fuzz/` (convite, frames, abertura de bytes) rodam com o cargo-fuzz, que precisa de
nightly:

```sh
cargo +nightly fuzz run frame_decode -- -max_total_time=60
```

A CI roda 60 s por alvo quando `src/room.rs`, `src/proto.rs` ou `fuzz/` mudam, e 30 min por semana.

### Testes de mutação e cadeia de suprimentos

```sh
cargo mutants --features test-network --file src/room.rs   # mutantes sobreviventes = comportamento sem teste
cargo deny check                                           # vulnerabilidades, licenças e fontes (deny.toml)
cargo bench                                                # cifra, frames e convites (criterion)
```

O cargo-mutants roda semanalmente na CI (`mutants.yml`); o cargo-deny roda em todo push.

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
