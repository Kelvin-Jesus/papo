# Trabalhando com o mantenedor

Preferências observadas e pedidas pelo mantenedor. Valem para pessoas e agentes.

- **Idioma.** O mantenedor escreve em português do Brasil: responda em pt-BR. Saída da CLI, README,
  livro, ADRs, wiki e `CONTEXT.md` em pt-BR. Código, comentários e o que o agente lê (instruções do
  servidor MCP, descrições e respostas das ferramentas, `knowledge/`) em inglês.
- **`AGENTS.md`, não `CLAUDE.md`.** Instruções para agentes ficam em `AGENTS.md`, para não amarrar o
  projeto a um fornecedor. Não crie `CLAUDE.md`.
- **Memória.** O destino padrão de memória durável é o ai-memory, no projeto `papo`. Sessões abertas na
  pasta pai (o workspace) resolvem para outro projeto; nesse caso, escreva com o projeto explícito.
  Nada de memória de sessão no repositório.
- **Autonomia.** Espere trabalhar sozinho por longos trechos. Junte as perguntas que precisam do
  mantenedor numa só, logo no começo; escolha padrões sensatos para o resto e diga quais escolheu.
- **O que precisa de "sim" antes:** publicar (push em `main` publica o site e o livro; uma tag publica
  um release), tornar algo público, mudar visual que vai para o ar (ele acompanha a saga de UI/UX no
  Claude Design antes), criar repositórios ou contas, apagar dados. Senhas, PINs e configurações do
  sistema são sempre dele.
- **Commits por partes.** Um assunto por commit, cada um compilando e passando nos testes, em
  Conventional Commits em português (`feat(node): ...`, `docs(engenharia): ...`), terminando com o
  trailer de coautoria. Commite só caminhos explícitos ([Agentes em paralelo](agentes-em-paralelo.md)).
- **Binários nativos.** Ferramentas do projeto saem como binário nativo (Rust; Go seria a alternativa),
  sem runtime para instalar.
- **Nomes.** Pronunciáveis e memoráveis em português e em inglês. O projeto já foi renomeado uma vez
  por isso.
- **Qualidade.** Testes e documentação andam com o código; diagramas em Mermaid são bem-vindos; o
  ginga, outro projeto do mantenedor, é a referência de rigor para documentação e de personalidade
  para site e marca.
- **Honestidade sobre verificação.** Diga o que foi verificado de verdade (internet, sessão real do
  Claude Code), o que só em teste com rede local e o que não foi verificado. A tabela de
  [Status](../engenharia/status.md) existe para isso.
- **Acompanhar o andamento.** Ele gosta de ver o trabalho evoluir: mostre prévias e relatórios curtos
  no caminho, não só no fim.
