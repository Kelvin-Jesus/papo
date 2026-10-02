# Ambiente de desenvolvimento

Peculiaridades das máquinas e ferramentas em uso, para não descobrir de novo.

- **Rust do gerenciador de pacotes, sem rustup.** A máquina principal de desenvolvimento usa o Rust do
  Arch Linux: só o alvo nativo está instalado, e não há `rustup component add`. Builds para macOS,
  Windows, ARM64 e musl saem do CI (`release.yml`, com `cargo zigbuild` para musl). Ferramentas que
  pedem componentes do rustup (como `llvm-tools-preview` para cobertura) rodam no CI ou com o LLVM do
  sistema.
- **Instale ferramentas fora do sistema.** Binários auxiliares (mdBook, cargo-llvm-cov) vão para uma
  pasta temporária ou de rascunho (`cargo install --root <pasta>` ou o tarball do release), não para o
  sistema.
- **mdBook 0.5.4.** A mesma versão fixada no workflow `pages`; baixe o tarball do release do
  rust-lang/mdBook. Versões diferentes podem gerar avisos diferentes.
- **Chromium.** Usado sem interface para validar diagramas (`scripts/check-mermaid.py`) e tirar prints
  do site. Precisa de internet para buscar o Mermaid no jsdelivr.
- **GitHub CLI.** `gh` autenticado com o escopo `repo` cobre push, workflows, Pages e releases. Para a
  wiki não há API.
- **Logs.** `PAPO_LOG` com filtros do `tracing-subscriber`; tudo no stderr. Os filtros úteis estão em
  [Desenvolvimento](../engenharia/desenvolvimento.md#diagnóstico-de-rede).
- **Perfis de teste.** `PAPO_HOME=/tmp/<algo>` para não tocar no `~/.papo` de verdade.
- **Servidores MCP na sessão.** A sessão do mantenedor tem ai-memory (memória durável) e
  codebase-memory (grafo do código) como servidores MCP. O codebase-memory precisa de
  `index_repository` antes da primeira consulta num repositório novo.
- **Claude Code.** Testado com a versão 2.1.285. Channels exige conta claude.ai ou chave do Console.
