# Agentes em paralelo

Várias sessões do Claude costumam trabalhar no papo ao mesmo tempo: uma no código, uma nos testes,
uma no site, uma na documentação. Estes são os hábitos que preservam o trabalho dos outros.

- **Worktree por frente grande.** Trabalho longo e isolado (uma suíte de testes, uma camada de
  documentação) roda num worktree git em `.claude/worktrees/` (ignorado pelo git), num branch próprio,
  com commits por partes. Quem coordena integra depois com rebase em `main`.
- **A sessão precisa estar dentro do repositório para criar o worktree.** Se ela foi aberta na pasta
  pai (o workspace, que não é um repositório git), a criação falha; entre no repositório antes.
- **Posse de arquivos.** Todo agente delegado recebe uma lista do que pode e do que não pode tocar,
  nomeando os arquivos de outras frentes. Na dúvida, não mexa: avise.
- **Olhe antes de commitar.** `git status` e `git log -3` antes e depois; a árvore muda por baixo de
  você. Arquivo que você não tocou é trabalho em andamento de alguém.
- **Commite caminhos explícitos.** `git add <arquivos>` e depois `git commit`. Nunca `git add -A`,
  nunca `git commit -a`, nunca uma pasta inteira onde outra frente está editando.
- **Nunca restaure arquivos que você não criou** (`git checkout -- f`, `git restore`). E nunca use
  `git stash` puro: a pilha de stash é compartilhada entre todos os worktrees. Prefira um commit
  temporário.
- **Builds paralelos.** Dois builds no mesmo `target/` esperam um pelo outro. Uma frente longa pode usar
  `CARGO_TARGET_DIR` próprio (custa compilar as dependências de novo, uns 5 minutos em release).
- **Portas e processos.** Servidores locais (mdBook, `http.server` para validar diagramas) em portas
  diferentes por frente. Não mate processos que você não iniciou.
- **Mensagens entre sessões.** Quando o seu trabalho chegar, sobrepuser o de outra frente ou você ver
  um problema na área de outro, avise com o caminho do arquivo e o sintoma exato. Não corrija o
  arquivo dos outros.
- **Delegando.** O agente delegado recebe um briefing autossuficiente: objetivo, arquivos permitidos e
  proibidos, como verificar, se pode ou não commitar. Revise o relatório dele e rode a verificação antes
  de integrar.
