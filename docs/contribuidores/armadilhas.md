# Armadilhas

O que já fez alguém perder tempo. As do produto têm a evidência técnica completa em
`knowledge/gotchas/` e a regra em [Problemas conhecidos](../engenharia/problemas-conhecidos.md); aqui
fica o resumo para quem está com a mão no código, mais as armadilhas de ferramenta.

## No produto

- **iroh-gossip não disca de novo.** Um par passado como bootstrap (ou por `join_peers`) cuja primeira
  discagem falha fica `Pending` para sempre. Nunca entregue pares ao gossip antes de existir uma
  conexão: o `node::connect_peer` disca com o ALPN do gossip e passa a conexão por
  `Gossip::handle_connection` ([ADR 0003](../adr/0003-o-papo-disca-os-pares-antes-do-gossip.md)).
- **Channels descartados em silêncio.** Se o Claude Code não foi aberto com a flag de channels (ou a
  organização não habilitou), as notificações somem sem erro. Por isso uma mensagem empurrada continua
  não lida até `wait`, `inbox` ou uma resposta com `reply_to`. Não "otimize" isso marcando como lida no
  push.
- **O stdout do `papo mcp` é do JSON-RPC.** Um `println!` ou um log no stdout corrompe o protocolo e o
  Claude Code derruba o servidor. Diagnóstico vai para o stderr (`eprintln!`, `tracing`).
- **A trava do perfil.** Só um `papo mcp` por perfil. Uma segunda sessão do Claude Code no mesmo perfil
  recebe "another papo server is already running": é o comportamento certo. Para testar dois agentes,
  use dois perfis ou dois `PAPO_HOME`.
- **Versão do protocolo MCP.** Não negocie revisões mais novas que `SUPPORTED_PROTOCOLS[0]`; o Claude
  Code não registra como channel quem negocia `2026-07-28`.
- **Chaves do `meta`.** Só `[A-Za-z0-9_]`; uma chave com hífen some do evento sem aviso.

## Nas ferramentas

- **Hook local bloqueia o `Read` em código.** Na máquina do mantenedor, um hook
  (`cbm-code-discovery-gate`) bloqueia a ferramenta `Read` em arquivos de código e manda usar o
  codebase-memory primeiro. Para ler um arquivo de código inteiro, use `sed -n` ou `cat` pelo shell, ou
  as ferramentas do codebase-memory.
- **RTK reescreve comandos.** O shell da máquina passa comandos como `git` pelo `rtk`. Num worktree
  isolado, a proteção de isolamento recusa `git` reescrito; chame `/usr/bin/git` direto. Para a saída
  crua de qualquer comando, `rtk proxy <comando>`.
- **Worktree a partir da pasta pai falha.** Ver [Agentes em paralelo](agentes-em-paralelo.md).
- **Filtro de caminho no primeiro push.** O workflow `pages` tem filtro de caminhos; no push que criou o
  branch `main` ele não disparou. Depois de criar um repositório, dispare à mão
  (`gh workflow run pages.yml`).
- **Pages precisa estar ligado antes do deploy.** O GitHub Pages tem de estar configurado com origem
  "GitHub Actions" (`gh api -X POST repos/<dono>/papo/pages -f build_type=workflow`) antes da primeira
  execução do `pages`.
- **A wiki só existe depois da primeira página.** O workflow `wiki` falha até alguém salvar uma página
  pela interface; não há API para isso.
- **`README.md` dentro do livro.** O mdBook transforma `README.md` em `index.html`, mas um link para
  `README.md` em outra página vira `README.html` e quebra. Aponte para a pasta (`adr/`).
- **Rótulos em `stateDiagram`.** O rótulo de uma transição vem depois de `:` e não pode ter outro
  (`Endpoint::connect` quebra o diagrama inteiro, e o livro não avisa).
- **`pgrep -f` pega o processo errado.** Com `timeout` ou `tail` no pipeline, o padrão casa com eles
  também; use `pgrep -x papo`.
- **Chromium headless e páginas altas.** Uma captura de 1280×2200 de uma página com diagramas chegou a
  ser morta por falta de memória; para validar diagramas use o `--dump-dom` do script, não a captura.
