# Hábitos do projeto

O que fazer antes de dizer que uma mudança está pronta. A lista curta de verificação está na skill
`papo-dev` (`.agents/skills/papo-dev/`); aqui está o porquê.

- **Meça antes de afirmar.** "Está mais rápido" vem com o número, a data e o comando
  ([Desempenho](../engenharia/desempenho.md)). "Funciona" vem com o teste que mostra.
- **Teste com o sistema de verdade.** Os testes de integração sobem endpoints iroh e gossip reais numa
  rede local; prefira esse estilo a mocks. Um teste que passa com mock e falha com a rede real está
  errado. O que precisa de internet fica `#[ignore]` e roda no CI como informativo.
- **O e2e público é o juiz da rede.** O bug de rediscagem do iroh-gossip passou em todos os testes
  locais e só apareceu com dois processos na internet. Mudou algo em `node.rs` ou `net.rs`? Rode
  `cargo test --test mcp -- --ignored`.
- **Documentação no mesmo commit.** A tabela "Keep in sync" do `AGENTS.md` diz o que atualizar para
  cada tipo de mudança (referência da CLI, das ferramentas MCP, do protocolo, `knowledge/`, ADR).
- **Decisão nova, ADR nova.** Se a mudança contraria uma ADR, escreva outra que a substitua; não edite
  a antiga a ponto de mudar o que ela decidiu.
- **Status honesto.** Mudou o estado de algo (verificado, testado, não verificado)? Atualize
  [Status](../engenharia/status.md) com a evidência.
- **Diagrama alterado, diagrama renderizado.** `python3 scripts/check-mermaid.py <arquivos>` antes do
  commit ([Desenvolvimento](../engenharia/desenvolvimento.md#validar-diagramas)).
- **Livro alterado, livro compilado.** `mdbook build docs` sem avisos e
  `scripts/check-book-links.py` sem problemas.
- **Saída limpa.** `cargo fmt`, `cargo clippy --all-targets -- -D warnings` e `cargo test` antes de
  cada commit de código; o CI roda o mesmo nos três sistemas.
- **Mensagens de commit por heredoc.** Mensagens com várias linhas vão por `git commit -F -` com
  heredoc; `$'\n'` dentro de aspas duplas não é interpretado e quebra a mensagem.
