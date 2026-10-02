# Pesquisa de UI/UX para o site do papo

Objetivo: dar personalidade ao site (https://kelvin-jesus.github.io/papo/) sem perder clareza, usando
como régua o site do ginga (https://kelvin-jesus.github.io/ginga/).

Data: 2026-10-02.

**Método**

- 21 sites estudados ao vivo com Chromium headless: captura 1440x900, captura alta 1440x4000 (ou
  página inteira com rolagem via Playwright quando o conteúdo só aparece ao rolar) e 390x844 com user
  agent de iPhone. As capturas ficaram no scratchpad da sessão, fora do repositório; o texto cita cada
  uma pelo nome do arquivo, por exemplo `[bun-tall]`.
- HTML e CSS publicados foram baixados para extrair famílias tipográficas (`@font-face`), `@keyframes`,
  curvas `cubic-bezier`, durações, número de menções a `prefers-reduced-motion` e bibliotecas. Os valores
  de movimento citados aqui vêm desse CSS, não de estimativa visual.
- Fatos do produto conferidos em `README.md`, `CONTEXT.md`, `docs/tutorial.md`, `docs/faq.md`,
  `docs/seguranca.md`, `docs/adr/0005-*.md`, `src/mcp.rs`, `src/node.rs` e `src/main.rs`.

---

## 1. Resumo: os princípios mais fortes, em ordem

1. **Um conceito só, levado a tudo.** Os sites que ficam na memória têm uma metáfora que vira sistema:
   ginga (Galaxy Tab vira cosmos: órbita, cometa, buraco negro, astronauta), Railway (trilho: hero com
   trem-bala e um trilho vertical que costura as seções), Charm (doceria: Bubble Tea, Lip Gloss,
   Glamour, mascotes de bala), PostHog (o site é um desktop de sistema operacional com ouriços). Para o
   papo, o conceito natural é a **sala de bate-papo**, porque cada convenção de mensageiro já
   corresponde a um fato do protocolo: sala, convite, "entrou na sala", online, visto por último, fila,
   entregue, responder, falar com todos ou só com alguém.
2. **O produto se demonstra sozinho, e com a semântica real.** Bun reproduz o benchmark "em tempo real"
   e tem um terminal simulado clicável; ginga simula Mac e tablet com uma máquina de estados de quatro
   passos e ramos (Wi-Fi ou cabo); Warp mostra uma grade "ao vivo" com legenda de estados; Ghostty faz
   do hero uma janela do próprio terminal. Demo que inventa comportamento destrói confiança: os estados
   mostrados têm de ser os estados do código.
3. **Um movimento-assinatura, ligado à função.** Cometa e pulso no cabo (ginga) mostram o transporte;
   o mascote do Bun pula quando vence o benchmark; o fantasma ASCII do Ghostty respira. Um gesto próprio
   repetido vale mais que dez fade-ins genéricos. No papo, o gesto é o **vai-e-volta**: a mensagem
   atravessa o fio e o recibo volta.
4. **Tipografia com opinião.** Nenhum site com personalidade usa `system-ui` como voz: Bun (Archivo
   condensada e pesada), Railway (IBM Plex Serif em "Ship software peacefully"), Fly (Mackinac), ginga
   (Unbounded larga), Zed (serif itálica azul). O site atual do papo usa `system-ui` e some no meio dos
   outros.
5. **Voz curta, com reframe, e humor nos detalhes.** Manchetes que viram o problema do avesso ("IP
   addresses break, dial keys instead", "Take shortcuts, not detours", "Ship software peacefully"). A
   graça mora nos lugares pequenos: selos do Charm ("Flavor | Taro"), banner de cookie da PostHog,
   haiku no rodapé do htmx.
6. **Prova concreta e honesta acima de adjetivo.** Gráfico com metodologia e link "reproduce" (Bun),
   0,06 s contra 4,63 s (uv), custo por mês (iroh), tabela comparativa datada (ginga). As provas do
   papo são verificáveis no código: fila gravada em disco antes do envio, ack só depois de gravar na
   inbox, freio de 40 envios em 10 minutos, XChaCha20-Poly1305, CI em Linux, macOS e Windows.
7. **Instalação como protagonista.** Comando copiável, abas de sistema e "ver o script" (Bun, Claude
   Code, Cursor). O papo não tem instalador por script; o equivalente honesto é detectar o sistema,
   oferecer o arquivo certo da release e o `cargo install`.
8. **Pouca cor, e com significado.** Base neutra mais uma ou duas cores que querem dizer algo: Astral
   (berinjela e lima), Bun (preto e rosa), ginga (cosmos, cobalto e uma estrela amarela só para
   destaque). O logo do papo já define o código: violeta é um lado da conversa, verde-azulado é o outro.
9. **Microcopy que convida a mexer.** "passe o mouse · clique no horizonte" (ginga), "click one, or
   just watch" e "Hover to pause" (Bun). Sem convite, ninguém descobre que o demo é interativo.
10. **Acessibilidade e performance como parte da identidade.** O ginga reduz todos os timers a 60 ms
    com `prefers-reduced-motion` e pausa o vídeo fora da tela; o CSS da página do Claude Code menciona
    reduced-motion 30 vezes. Um site estático no GitHub Pages pode ser mais rápido e mais respeitoso que todos.

---

## 2. Análise por site

Campos: ideia memorável, hero, demonstração, tipografia, cor, movimento, voz, o que roubar, o que
evitar.

### 2.1 Ferramentas de dev com personalidade

**charm.sh** `[charm-desktop] [charm-tall] [charm-mobile]`

- Ideia: "We make the command line glamorous". Doceria em 3D: cada biblioteca tem mascote e nome de
  doce (Bubble Tea, Lip Gloss, Glamour, Harmonica).
- Hero: bolhas rosa e violeta com brilhos; card com o produto (Crush) e um coração em voxel; contador
  de estrelas do GitHub (222,7k) no canto.
- Demo: GIFs/vídeos de TUIs gravados com VHS dentro dos cards.
- Tipo: Anchor e Mori (texto), JetBrains Mono; logo em script próprio.
- Cor: violeta saturado e rosa-chiclete.
- Movimento: brilhos, `float` nos mascotes, `cursor-blink`, `bg-colorcycle`. Nenhuma menção a
  reduced-motion.
- Voz: selos de duas partes com piada: "Flavor | Taro", "Glossiness | Very", "Type | Tapioca",
  "Magical | Yes".
- Roubar: selo de duas partes como lugar da piada; no celular, botão "menu" em forma de adesivo na
  zona do polegar.
- Evitar: textura, saturação e 3D em excesso; nenhum cuidado com reduced-motion.

**htmx.org** `[htmx-tall]`

- Ideia: anti-hype. Página de texto quase sem design, logotipo `</> htmx` e um haiku fixo no rodapé
  ("javascript fatigue: / longing for a hypertext / already in hand").
- Hero: logotipo enorme sobre textura topográfica; "high power tools for HTML".
- Demo: seis linhas de código no quick start e uma citação explicando o que o botão faz.
- Tipo: só fonte do sistema. A personalidade vem inteira da voz.
- Voz: provocadora, em perguntas ("Why should only `<a>` & `<form>` be able to make HTTP requests?").
- Roubar: motivação em perguntas retóricas; uma piada fixa e discreta no rodapé.
- Evitar: minimalismo extremo só funciona para projeto famoso com tese polêmica.

**posthog.com** `[posthog-desktop] [posthog-tall]`

- Ideia: o site é um desktop (ícones nas laterais, janela com botão de fechar, lixeira) com ouriços
  ilustrados em cada seção.
- Hero: "Your product's context layer" numa janela; card "Set up for free" com checklist ("97% of users
  pay us $0"); abas com demo animado e botão de pausa.
- Tipo: IBM Plex Sans Variable, fontes próprias (RoundHog, Squeak), Source Code Pro.
- Cor: bege e verde claro, laranja e amarelo nos botões, abas coloridas.
- Movimento: dezenas de animações nomeadas (`inbox-arrival` 6,4 s em `cubic-bezier(.22,1,.36,1)`,
  `hogfather-jump`); 18 menções a reduced-motion.
- Voz: autodepreciativa ("Shameless CTA", "Bedtime reading", "Legally-required cookie banner"; toggle
  "Colorful logos / Sleek logos").
- Roubar: títulos de seção com voz; honestidade explícita; demo com pausa.
- Evitar: densidade de piadas; HTML de 1,6 MB.

**railway.com** `[railway-desktop] [railway-tall]`

- Ideia: trilho. Ilustração noturna com montanha e trem-bala; um trilho vertical à esquerda costura as
  seções, com um vagão como marcador.
- Hero: "Ship software peacefully" em serif; dois botões; canvas do produto com logs de build reais e
  toast "Deployment Successful".
- Tipo: IBM Plex Serif (display), Inter e Inter Tight (texto), JetBrains Mono, todos do Google Fonts.
- Movimento: `agent-waking-train`, `balloon-float`, marquee de logos.
- Voz: calma, com uma palavra inesperada ("peacefully").
- Roubar: o indicador de progresso é a própria metáfora; serif para dar calma a um produto técnico.
- Evitar: banner de cookies cobrindo o hero.

**fly.io** `[fly-desktop] [fly-tall]`

- Ideia: ilustração à mão com aquarela: servidores com asas e bico, balão do logo, pássaros.
- Hero: "Computers for agents" em Mackinac; "Sandboxes aren't enough."
- Tipo: Mackinac (serif), Fricolage Grotesque, Fragment Mono.
- Cor: pastéis de céu e roxo vivo nos botões.
- Movimento: discreto (pontinhos de loading de 0,6 s).
- Roubar: desenho à mão dá calor a infraestrutura.
- Evitar: ilustração encomendada sem artista vira clipart.

**ghostty.org** `[ghostty-desktop] [ghostty-tall]`

- Ideia: o hero é o produto. Uma janela do Ghostty rodando um fantasma em ASCII animado; abaixo, uma
  frase e dois botões. Nada mais.
- Tipo: Pretendard e JetBrains Mono.
- Movimento: quadros de texto ASCII.
- Roubar: deixar o meio do produto (texto em terminal) ser a arte; coragem de ter uma coisa só.
- Evitar: na captura alta o resto da página é vazio; não há prova além do visual.

**astral.sh e docs.astral.sh/uv** `[astral-desktop] [astral-tall] [uv-desktop]`

- Ideia: "ferramenta do futuro": berinjela profunda, lima elétrica no CTA, logotipo pixelado, caixa
  alta.
- Demo: na doc do uv, um gráfico de barras é a primeira coisa depois do título (uv 0,06 s; poetry
  0,99 s; pdm 1,90 s; pip-sync 4,63 s).
- Tipo: Alliance Platt e Alliance Text, Roboto Mono.
- Roubar: uma prova numérica no topo; docs com a mesma marca do site.
- Evitar: hero gigante sem conteúdo (ocupa quase toda a captura de 4000 px); caixa alta demais cansa.

**bun.sh** `[bun-desktop] [bun-tall] [bun-mobile]`

- Ideia: velocidade provada ao vivo. Gráfico de instalação com "replay in real time", o mascote
  montado na barra vencedora (bun 0,21 s; yarn 1,76 s; pnpm 1,92 s; npm 4,45 s), nota de metodologia e
  link "reproduce".
- Hero: H1 em Archivo condensada pesada com "fast" em rosa; `curl` copiável com abas "macOS & Linux |
  Windows" e "View install script".
- Demo: "A minute with Bun": cinco passos clicáveis e um terminal que toca saída real, com contador
  "01 / 05", replay e "Hover to pause".
- Tipo: Archivo (variável, larguras condensadas) e Martian Mono.
- Cor: preto, cinzas e rosa-choque; selos "starburst" coloridos ("REPLACES Node.js") girando devagar.
- Movimento: `row-in` 0,7 s em `cubic-bezier(.16,1,.3,1)`; `hop` 0,6 s do mascote; 6 menções a
  reduced-motion.
- Roubar: abas de sistema, "ver script", metodologia, terminal clicável com contador e pausa, selos
  como adesivos.
- Evitar: barra de anúncio no topo; parede de logos.

**zed.dev** `[zed-part-0]`

- Ideia: elegância editorial: "Your last next editor" em serif itálica azul, e atalhos de teclado nos
  próprios botões ("Download now [D]", "Clone source [C]").
- Tipo: IBM Plex Sans e Serif, iA Writer Quattro, Lilex.
- Movimento: `hero-rise`, `hero-drop`; mola `cubic-bezier(.34,1.56,.64,1)`.
- Roubar: atalho de teclado visível no CTA (o público vive no teclado); depoimentos de gente que o
  público respeita.
- Evitar: grade de screenshots genérica no meio da página.

**warp.dev** `[warp-desktop] [warp-tall]`

- Ideia: prancheta técnica: legendas "[ fig. 1 — the factory ]", texto vertical nas margens ("WARP ·
  HOME · 2026"), grade de quadradinhos "LIVE" com legenda de estados (queued, working, waiting,
  shipped) e contadores.
- Demo: visualização de estados. No CSS há um mini-chat com digitação (`mini-app-typing-bounce` 1,2 s)
  e entrada de mensagem (0,24 s em `cubic-bezier(.16,1,.3,1)`), e uma presença em texto mono que
  avança de " [.]" para " [..]", " [...]" e " [online]" com `step-end`.
- Tipo: theFuture e Matter (mono), Azeret Mono, Instrument Serif, Inter.
- Movimento: muito; 11 menções a reduced-motion; botão de som no canto.
- Roubar: legenda de estados colada na visualização; presença em texto; legendas de figura.
- Evitar: jargão corporativo; som.

**linear.app** `[linear-desktop] [linear-tall]`

- Ideia: polimento e contenção: UI do produto em alta fidelidade, monocromia, frase em duas cores
  (branco e cinza continuando).
- Tipo: Inter Variable.
- Movimento: grade de pontos com centenas de keyframes gerados.
- Roubar: frase em duas cores; nada fora do lugar.
- Evitar: o "estilo Linear" (preto, cinza, Inter, brilho) é exatamente a origem do site genérico de
  hoje.

**raycast.com** `[raycast-part-0] [raycast-part-1]`

- Ideia: atalho. Listras vermelhas 3D no hero, "Your shortcut to everything", e um mock de macOS com
  dock de recursos que troca o conteúdo da janela.
- Demo: janelas por recurso; cards de extensões com mini-demos vivos (tradução rolando, player).
- Tipo: Inter, Instrument Serif, Geist Mono, JetBrains Mono, VT323.
- Movimento: 103 usos de 0,3 s; curvas `(.23,1,.32,1)` e `(.16,1,.3,1)`; 8 menções a
  reduced-motion.
- Voz: trocadilhos ("Take shortcuts, not detours"; "Stop playing Clipboard ping pong"; "It's not about
  saving time. It's about feeling like you're never wasting it.").
- Roubar: "Clipboard ping pong" é exatamente a dor que o papo resolve; mini-demo dentro de card.
- Evitar: hero 3D pesado.

### 2.2 Redes P2P: explicar uma coisa invisível

**tailscale.com** `[tailscale-tall]`

- Hoje é plataforma enterprise: abas por caso de uso, screenshot do app, números de clientes. O
  diagrama de rede que explicava o produto virou screenshot.
- Instalação mostrada como três linhas de terminal ao lado do resultado (login e lista de máquinas).
- Movimento: `draw-check`, um check que se desenha.
- Roubar: check desenhado como confirmação; instalação lado a lado com o efeito.
- Evitar: a deriva para "enterprise genérico".

**iroh.computer** `[iroh-desktop] [iroh-tall]`

- Ideia: reframe técnico numa linha: "IP addresses break, dial keys instead".
- Demo: diagrama com nó central e caminhos (nuvens, self-host, edge); tabela de custo (cerca de
  $1.742/mês centralizado contra cerca de $100/mês).
- Tipo: Space Grotesk e Space Mono.
- Roubar: a frase-reframe; custo como prova.
- Evitar: Space Grotesk e o lilás do iroh. O papo usa iroh e precisa parecer outra coisa.

**localsend.org** `[localsend-tall]`

- Ideia: "Share files without the cloud. Fast, private, offline." O aparelho recebe nome aleatório
  simpático ("Fast Papaya #109").
- Demo: "How it works" animado: o arquivo sai de um lado, viaja e entra no outro, com check de
  sucesso (`sourceFileOut`, `transfer`, `targetFileIn`, `showSuccess`, laço de 2,5 a 3 s).
- Prova: 70k+ estrelas, 5M+ downloads, 100+ contribuidores, "0 ads or trackers".
- Roubar: "sai daqui, chega lá, check"; "zero rastreadores" como número.
- Evitar: grade de quatro features com ícone.

**magic-wormhole e croc** `[wormhole-desktop] [croc-tall]`

- Ideia: o código humano é a interface (wormhole: "Get things from one computer to another, safely",
  códigos como `7-crossover-clockwork`). O README do croc mostra dois terminais empilhados, "sender >"
  e "receiver >", fazendo a mesma transferência dos dois lados, e um crocodilo desenhado à mão.
- Roubar: **os dois lados na mesma tela, ao mesmo tempo**. Para P2P isso explica mais que qualquer
  diagrama. Tratar o código/convite como objeto protagonista.
- Evitar: README como site; anúncio no meio do texto.

**syncthing.net** `[syncthing-desktop]`

- Prosa clara e honesta ("The protocol is a documented specification, no hidden magic") num visual
  Bootstrap com Font Awesome. É o retrato de como "sem personalidade" envelhece.
- Roubar: a franqueza do texto. Evitar: todo o resto.

### 2.3 Produtos de agentes e mensageiros

**claude.com/product/claude-code** `[claudecode-desktop] [cc-part-0] [cc-part-1]`

- Ideia: serif da marca, fundo quase preto e quente, laranja-terracota, ícones desenhados à mão
  (broto, caixa de ferramentas).
- Hero: título serif, botões largos, janela do app com uma sessão real (bug de cobrança duplicada) e a
  fileira "Other ways to use Claude Code".
- Demo: abas (Onboarding, Triage issues, Migrate code) com terminal; o terminal mostra os verbos de
  espera ("Simmering…", "Pondering…"), que viraram assinatura do produto.
- Movimento: `row-dot-wave` 1,05 s (opacidade .25 a 1); digitação em CSS puro com
  `steps(var(--chars))`; spinner em sprite com `steps()`; 30 menções a reduced-motion.
- Roubar: verbos de espera com personalidade (o público do papo já gosta dos do Claude Code); digitação
  em CSS; comando de instalação repetido no meio da página.
- Evitar: imitar a identidade da Anthropic (fonte, laranja, asterisco). O papo é independente e o
  rodapé atual diz isso; o visual também precisa dizer.

**cursor.com** `[cursor-part-0] [cursor-part-1]`

- Ideia: UI de agente sobre pinturas a óleo de paisagem; a página veio em pt-BR sozinha pelo idioma do
  navegador.
- Demo: janelas do app, CLI e Slack com conversa em andamento; passos do agente com duração ("Read 3
  files, 1 search 1s", "Planned 2s", "Spawning 3 agents") e pontos cheios ou vazados.
- Movimento: `streaming-word-fade` de 80 ms por palavra (opacidade .4 a 1); entrada de chat 0,42 s em
  `cubic-bezier(.22,1,.36,1)`.
- Roubar: fade curto por palavra para simular streaming; lista de passos com tempo; localização.
- Evitar: muro de depoimentos de CEO.

**Como mensageiros sinalizam estado** (referência para o demo do papo)

| App | Enviando | Saiu do aparelho | Entregue | Lido | Digitando |
| --- | --- | --- | --- | --- | --- |
| WhatsApp | relógio | um check cinza (chegou no servidor) | dois checks cinza | dois checks azuis | "digitando…" no topo |
| Signal | círculo pontilhado girando | círculo com check (servidor) | dois círculos com check | dois círculos preenchidos, se ambos ligarem recibos | bolha com três pontos |
| iMessage | barra de progresso | - | "Entregue" sob a última bolha | "Lido 14:03" | bolha com três pontos que respira |

O que esses apps ensinam:

- O estado mora **na própria bolha**, pequeno, ao lado do horário. Texto por extenso ("Entregue") só na
  última mensagem.
- A troca de estado é discreta: troca de ícone, sem animação longa.
- Indicador de digitação: três pontos, ciclo de 1 a 1,4 s, atraso de 160 a 180 ms entre pontos,
  animando só `transform` e `opacity`.

**Mapeamento honesto para o papo** (conferido em `src/node.rs`, `src/mcp.rs`, `src/main.rs`):

- Não há servidor, então não existe o "check único" de "chegou no servidor".
- O `send` termina em um de dois estados: **entregue** (ack de quem recebeu, que só responde depois de
  gravar na própria inbox; o `send` espera até 8 s) ou **na fila** (a mensagem já está na outbox em
  disco; é reenviada quando um vizinho conecta e a cada 30 s; o log registra `delivered <id> to <nome>`
  quando o ack chega).
- **Não existe "lido" para quem enviou.** O que existe é `reply_to`: a resposta aponta a mensagem que
  responde. O demo pode mostrar "respondida" como citação, nunca como dois checks azuis.
- `papo say` não enfileira: sem ninguém online, falha com "ninguém da sala está online agora; a
  mensagem não foi enviada".
- Online = vizinho direto ou alguém de quem se ouviu algo nos últimos 75 s. Offline mostra "visto há
  N min" (`describe_peer`).

### 2.4 ginga (benchmark interno) `[ginga-desktop] [ginga-tall] [ginga-mobile]`

Resumo aqui; decomposição completa na seção 4. Conceito cósmico derivado do nome do aparelho (Galaxy),
arte própria em pixel com dithering, demo-história clicável que replica as telas reais, tipografia
larga (Unbounded) com legenda mono amarela, tokens de movimento com nome, bilíngue, tema claro e
escuro, e reduced-motion levado a sério.

---

## 3. Padrões transversais

### 3.1 Hero

- Três formatos se repetem: (a) frase mais prova ao lado (Bun: H1, instalação e gráfico); (b) o produto
  é o hero (Ghostty, Linear, Claude Code, Cursor); (c) arte do conceito mais frase (ginga, Railway, Fly,
  Charm).
- Os melhores juntam: frase de até 9 palavras, subtítulo concreto, **um** CTA primário, e o produto ou
  a prova visível na primeira dobra a 1440x900.
- O hero atual do papo é do tipo (a), mas com chat estático e três botões de peso parecido.

### 3.2 Demonstração do produto

| Técnica | Exemplos | Custo | Quando serve |
| --- | --- | --- | --- |
| Screenshot estático | Linear, Zed, Tailscale | baixo | produto visual e já conhecido |
| Vídeo | ginga (30 s), Bun (depoimentos) | alto (peso), não interativo | complemento, nunca o único demo |
| UI simulada que se anima, com pausa | PostHog, Warp, Claude Code | médio | mostrar fluxo |
| Terminal simulado clicável | Bun ("A minute with Bun") | médio | CLI |
| Réplica com máquina de estados e ramos | ginga | médio-alto | o que mais fica na memória |
| Os dois lados ao mesmo tempo | croc, LocalSend | baixo | P2P: indispensável |

### 3.3 Interação

- Convite explícito para mexer, com o verbo certo ("clique", "passe o mouse", "derrube").
- Pausa, recomeço e contador de passos ("01 / 05").
- Ramo que demonstra uma propriedade (ginga: "Prefiro o cabo USB").
- Atalhos de teclado nos CTAs (Zed).
- Hover nunca é o único caminho: tudo que reage a hover também reage a toque e foco.

### 3.4 Tipografia

| Site | Display | Texto | Mono |
| --- | --- | --- | --- |
| Charm | logo próprio, Anchor | Anchor, Mori | JetBrains Mono |
| PostHog | IBM Plex Sans, RoundHog | IBM Plex Sans | Source Code Pro |
| Railway | IBM Plex Serif | Inter, Inter Tight | JetBrains Mono |
| Fly | Mackinac | Fricolage Grotesque | Fragment Mono |
| Ghostty | Pretendard | Pretendard | JetBrains Mono |
| Astral | Alliance Platt (caixa alta) | Alliance Text | Roboto Mono |
| Bun | Archivo condensada | Archivo | Martian Mono |
| Zed | IBM Plex Serif itálica | IBM Plex Sans, iA Writer Quattro | Lilex |
| Warp | theFuture, Matter | Inter | Azeret Mono |
| Linear | Inter Variable | Inter | mono do sistema |
| Raycast | Inter, Instrument Serif | Inter | Geist Mono, VT323 |
| iroh | Space Grotesk | Space Grotesk | Space Mono |
| Claude Code | anthropicSerif | anthropicSans | anthropicMono |
| Cursor | CursorGothic | CursorGothic | berkeleyMono |
| ginga | Unbounded | Figtree | IBM Plex Mono |
| **papo hoje** | system-ui | system-ui | ui-monospace |

Padrões:

- Três papéis: display com caráter, texto neutro, mono para dado, comando e legenda.
- Eixos variáveis como recurso expressivo: largura no Archivo (Bun), largura extrema no Unbounded
  (ginga).
- Legenda (kicker) em mono, caixa alta e espaçada, numa cor de destaque: ginga (amarelo-estrela), Bun
  (rosa, com selo "NEW"), Warp.
- Mono fora do código, como voz de "máquina": Warp, ginga, Claude Code.

### 3.5 Cor

- Base neutra, uma cor de ação e uma cor de estado. Exemplos: Astral (berinjela, lima), Bun (preto,
  rosa), ginga (cosmos `#0A0C1C`, cobalto `#2E47F5`, estrela `#FFC43D` só em legenda e brilho).
- Cores com nome do conceito (ginga: `--cosmos`, `--cobalt`, `--star`, `--stardust`) ajudam a manter
  a disciplina.
- Light e dark tratados como dois temas desenhados, não como inversão automática.

### 3.6 Movimento (valores medidos)

| Origem | O que anima | Duração | Curva e detalhe |
| --- | --- | --- | --- |
| Warp | entrada de mensagem | 240 ms | `cubic-bezier(.16,1,.3,1)`, sobe 6 px |
| Warp | três pontos digitando | 1,2 s em laço | ease-in-out, sobe 4 px aos 40% |
| Warp | presença em texto [.] [..] [...] [online] | 1,3 s | `step-end` |
| Cursor | palavra em streaming | 80 ms | ease-out, opacidade .4 a 1 |
| Cursor | entrada de mensagem (mobile) | 420 ms | `cubic-bezier(.22,1,.36,1)`, sobe 12 px |
| Claude Code | pontos "trabalhando" | 1,05 s em laço | opacidade .25 a 1 |
| Claude Code | digitação | n caracteres x passo | `steps(n, jump-start)` |
| Bun | linha entrando | 700 ms | `cubic-bezier(.16,1,.3,1)`, sobe 14 px |
| Bun | mascote pulando | 600 ms | ease-out com rotação de -10° e 4° |
| LocalSend | arquivo sai, viaja, chega, check | 2,5 a 3 s em laço | keyframes por porcentagem |
| PostHog | itens chegando na inbox | 6,4 s | `cubic-bezier(.22,1,.36,1)` |
| ginga | cor e sombra de UI | 220 ms | `cubic-bezier(.2,.8,.2,1)` |
| ginga | painel (sheet) | 420 ms | `cubic-bezier(.34,1.36,.64,1)` (passa do ponto e volta) |
| ginga | cometa entre aparelhos | 1400 ms | easeInOutQuad em `requestAnimationFrame` |
| ginga | pulso no cabo | 1000 ms | `cubic-bezier(.4,0,.2,1)` |
| ginga | onda de chegada | 700 ms | `cubic-bezier(.2,.8,.2,1)`, raio de 6 a 70 |

Convergências: `cubic-bezier(.16,1,.3,1)` é a saída padrão de quem anima bem (Bun, Warp, Raycast, Zed,
PostHog); um overshoot leve (`.34,1.36,.64,1` no ginga, `.34,1.56,.64,1` no Zed) dá o "pop"; UI fica
entre 120 e 240 ms, entradas de mensagem entre 240 e 420 ms, viagens entre 900 e 1400 ms, laços
ambientes entre 1 e 3 s.

### 3.7 Copy

- Reframe numa linha (iroh, Raycast).
- Uma palavra inesperada (Railway: "peacefully").
- A dor com nome próprio (Raycast: "Clipboard ping pong").
- Número com contexto e método (Bun, uv, iroh).
- Humor só em microcopy e rodapé; instrução e segurança sempre sérias.
- Idioma do visitante: Cursor localiza; ginga é bilíngue com escolha visível.

### 3.8 Credibilidade

- Comando de instalação copiável com abas de sistema e "ver script" (Bun, Claude Code, Cursor).
- Contador de estrelas só quando ajuda (Charm 222,7k, LocalSend 70k+). Hoje o repositório do papo tem
  0 estrelas, nenhuma tag e **nenhuma release publicada** (conferido com `gh api` em 2026-10-02). O
  botão "Baixar" do site atual leva a uma página de releases vazia.
- Fatos verificáveis no lugar de vaidade: licença MIT, Rust, CI em Linux, macOS e Windows, teste com
  dois servidores MCP pela internet real (`cargo test --test mcp -- --ignored`), código aberto.
- Metodologia junto do número (Bun: "reproduce"); comparação datada com fontes (ginga).

### 3.9 Entradas para a documentação

- Docs como extensão da marca (uv usa o tema do Astral; o livro do papo já tem `docs/theme/papo.css`).
- Links por intenção, não por estrutura: "Tutorial (10 min)", "Pedidos que funcionam", "Referência das
  ferramentas", "Segurança".

### 3.10 Mobile

- ginga põe a arte acima do título no celular e mantém os CTAs com largura total.
- Bun mantém as abas de sistema e deixa o comando rolar dentro do bloco.
- Charm usa um botão-adesivo fixo na zona do polegar.
- Demos de dois lados empilham na vertical; o diagrama atual do papo já tem versão vertical, bom ponto
  de partida.

---

## 4. O que faz o ginga ter personalidade

Decomposto em mecânicas reaproveitáveis. Fontes: `GingaSite.mFDliisY.js`, `Base.YWBjuY7q.css`,
`config.C722aY1T.css` e `ds/bundle.js` publicados.

1. **Trocadilho que vira universo.** "Galaxy" Tab vira cosmos e o cosmos aparece em tudo: arte do hero
   (buraco negro), títulos ("Três jeitos de entrar em órbita"), narrativa do demo ("Mac e tablet entram
   em órbita", "o sinal atravessa a rede como um cometa"), easter egg (astronauta). *Reuso:* escolher
   um conceito que gere vocabulário para títulos, estados e microcopy.
2. **Arte própria numa técnica única.** Canvas com dithering em pixel (o canvas roda a 1/4 da
   resolução e é ampliado), paleta de seis cores, usado no hero e também como papel de parede dos mocks
   do Mac e do tablet. Uma técnica, aplicada em vários lugares. *Reuso:* um elemento gráfico do papo
   (os dois anéis do logo) aparece no hero, no demo, nos estados e no fundo.
3. **Demo-história com máquina de estados.** Fases (`connecting`, `connected`...), quatro passos
   nomeados ("1 Ligar no Mac · 2 Encontrar · 3 Parear · 4 Estender"), legenda narrativa ("Passo 2 de 4
   · Tablet"), ramo alternativo ("Prefiro o cabo USB"), "Recomeçar", e réplicas fiéis das telas reais.
   *Reuso:* o demo do papo também é um roteiro com estados reais e ramos.
4. **Movimento que significa transporte.** Wi-Fi é um cometa (traço de 90 px correndo um caminho SVG
   em 1400 ms); USB é um pulso de luz no cabo (1000 ms); a chegada é uma onda que se abre (700 ms); a
   conexão faz um "warp" em canvas. *Reuso:* a mensagem do papo viaja pelo fio e o ack volta.
5. **Tokens de movimento com nome próprio.** `--ease-ginga` (overshoot), `--dur-orbit` 2,4 s,
   `--dur-warp` 1,4 s, `--dur-sheet` 0,42 s, `--dur-ui` 0,22 s, `--dur-tap` 0,12 s. *Reuso:*
   `--ease-papo`, `--dur-viagem`, `--dur-ack`.
6. **Tipografia em três papéis.** Unbounded larga e pesada no display, Figtree no texto, IBM Plex Mono
   na legenda amarela em caixa alta com uma estrela de quatro pontas e no bloco de código final.
7. **Paleta com nomes do conceito.** `--cosmos #0A0C1C`, `--cobalt #2E47F5`, `--star #FFC43D`,
   `--stardust #F2F3F8`. A estrela amarela é rara, por isso chama atenção.
8. **Microcopy-convite.** "passe o mouse · clique no horizonte" em mono sob a arte; "Experimente aqui
   embaixo: toque ou clique na tela do Mac e conecte o tablet". O botão escondido na arte tem nome
   acessível ("Disparar o buraco negro").
9. **Easter egg que não atrapalha.** O astronauta que segue o ponteiro só liga com `pointer: fine` e sem
   reduced-motion.
10. **Honestidade nos detalhes.** Números medidos (cerca de 230 mW, 13 ms), tabela comparativa com
    data ("setembro de 2026") e aviso de marcas, FAQ que admite limites ("Macs com Intel não são
    suportados"), compatibilidade com selos "testado" e "a confirmar".
11. **Acabamento de produto.** PT/EN com idioma na URL, tema claro e escuro, design system com
    componentes nomeados (`StatusOrbit`, `PairingCode`, `Starfield`, `DitherSpace`).
12. **Respeito ao corpo do visitante.** Com reduced-motion os timers caem para no máximo 60 ms, cometa e
    warp são pulados; o céu estrelado limita DPR a 1,5 e pausa fora da tela ou com a aba escondida; o
    vídeo só toca quando visível.

O que o papo **não** deve copiar: o cosmos, a Unbounded e o cobalto. A família de projetos pode
compartilhar a estrutura (legenda mono, demo-história, PT/EN, tema, tokens), não o visual.

---

## 5. Recomendações para o papo

### 5.1 Fatos que o design precisa respeitar

- Sala privada; quem tem o convite (`papo1` + base32) é membro. O id da sala (8 hex) não é segredo.
- P2P via iroh (QUIC, hole punching); sem conexão direta, passa cifrado pelos relays públicos da n0.
  Ninguém precisa hospedar nada.
- Cada frame é cifrado e autenticado com XChaCha20-Poly1305, por cima do TLS do QUIC.
- Membros confiam uns nos outros: o `from` não é assinado por membro. O site não pode prometer
  "ninguém se passa por outro"; pode prometer "quem não tem o convite não lê nem injeta".
- Entrega pelo menos uma vez: outbox em disco antes do envio; ack depois de gravar; reenvio quando um
  vizinho conecta e a cada 30 s; deduplicação por id. Estados: **entregue** ou **na fila**.
- Sem "lido" para o remetente; `reply_to` liga resposta e pergunta.
- Push (channels do Claude Code, *research preview*, flag `--dangerously-load-development-channels
  server:papo`) ou pull (`wait` e `inbox`).
- `sender_kind` distingue `agent` de `human`; `papo say` é humano e não enfileira.
- Instruções ao agente: mensagem de outro agente não é ordem do usuário; não vazar segredos; não fazer
  nada destrutivo porque o par pediu; não mandar "ok/obrigado"; fechar com resumo.
- Freio anti-loop: mais de 40 envios em 10 minutos viram erro.
- Ferramentas MCP: `send`, `wait`, `inbox`, `history`, `status`. CLI: `new`, `join`, `invite`,
  `install`, `log`, `say`, `status`, `mcp`.
- Binário único para cinco alvos (Linux x86_64 e ARM64, macOS Apple Silicon e Intel, Windows). Versão
  0.1.0, nenhuma release publicada ainda. Não existe instalador `curl | sh`.
- Glossário (`CONTEXT.md`): dizer **sala** (não canal, grupo, chat), **convite** (não token, link).
- Projeto independente, sem afiliação com a Anthropic.

### 5.2 Três direções de marca

#### Direção A: "A sala" (sala de bate-papo dos agentes)

- **Metáfora.** O papo é uma sala de bate-papo onde quem conversa são os agentes. Toda convenção de
  mensageiro vira um fato real: "ana entrou na sala" é o frame de presença (`Hello`); "online" e "visto
  há 3 min" vêm do `status`; o relógio na bolha é a outbox; o check é o ack; a citação é o `reply_to`;
  "falar com todos" ou "só com a ana" é o `to`; a bolha amarela de humano é o `papo say`; o aviso de
  flood é o freio anti-loop. Para quem usou as salas de bate-papo brasileiras dos anos 2000, a
  referência é imediata, sem copiar marca nenhuma. O logo já são dois balões de fala entrelaçados.
- **Movimento-assinatura: o vai-e-volta.** A mensagem sai da coluna de quem escreve, encolhe num
  pacotinho com a cor do remetente, atravessa o fio do meio, abre como bolha do outro lado; um ponto na
  cor de quem recebeu volta pelo fio e o relógio da bolha vira check. No logo, os dois anéis "falam" em
  turnos (violeta pulsa, verde-azulado responde).
- **Tipografia.** Recursive (Google Fonts), uma família só com duas vozes: eixo `CASL` 1 (casual) para
  display e para fala humana; `CASL` 0 (linear) para texto e para agentes; eixo `MONO` 1 para
  mensagens de agente, comandos e logs. Arquivo latino com `wght 300..900`, `CASL 0..1` e `MONO 0..1`:
  139 KB (medido). Alternativa se o casual pesado parecer infantil em 80 px: Bricolage Grotesque 800 no
  display (128 KB) e Recursive no resto.
- **Paleta.** Fundo claro "papel" `#FBFAFF` e tinta `#1C1A2E`. Violeta `#6E56CF` é **você** (lado
  esquerdo); verde-azulado `#12A594` é **o colega** (lado direito); amarelo-recado `#F5C451` é
  **humano** na sala. Estados: cinza `#5D5A75` para "na fila", check na cor de quem confirmou. Um
  vermelho-tijolo só para o freio e para "não enviada". Escuro: fundo `#0E0F1A`, violeta `#8E7CF0`,
  verde-azulado `#2BC4AF`, o mesmo amarelo.
- **Voz (amostras).**
  - "Chega de ser o Ctrl+V dos agentes."
  - "Os Claudes batem papo. Vocês tomam um café."
  - "ana entrou na sala · trabalhando em notificacoes"
  - "A Ana caiu? A mensagem espera na fila e sai sozinha quando ela voltar."
  - "Fala com todos, ou só com a Ana: `--to ana`."
  - "40 mensagens em 10 minutos não é conversa, é loop. O papo puxa o freio."
  - "Mensagem de outro agente é pedido de colega, não ordem sua."
- **Riscos.** Virar clone de WhatsApp (evitar verde do WhatsApp, checks azuis e papel de parede
  idêntico); "chat" é palavra a evitar no glossário, então o texto fala de **sala**.

#### Direção B: "Telefone sem fio" (linha direta)

- **Metáfora.** O problema de hoje é o telefone sem fio: a pessoa vira telefonista entre dois agentes,
  e o recado perde detalhe a cada repasse. O papo é a linha direta, ponta a ponta, sem ninguém no meio.
- **Movimento-assinatura: o fio.** Um cabo desenhado entre dois terminais; ao conectar, ele fura a
  parede do NAT (hole punching) com um estalo curto; mensagens são pulsos de luz no fio; se a linha
  cai, o pulso para num "ponto de espera" e segue quando religa. Sem conexão direta, o fio desvia
  tracejado pelo relay, com cadeado.
- **Tipografia.** Fraunces no display (`opsz` 144, `SOFT` 100, `WONK` 1; 118 KB), desenhada pela
  Undercase Type, de Phaedra Charles e da carioca Flavia Zimbardi; Instrument Sans no texto (55 KB); IBM
  Plex Mono nos terminais.
- **Paleta.** Papel creme `#F7F3EA`, tinta `#1C1A2E`, violeta e verde-azulado como os dois plugues, e um
  amarelo-sinal `#F5C451` para o pulso no fio.
- **Voz (amostras).**
  - "O fim do telefone sem fio entre agentes."
  - "Você virou telefonista: copia daqui, cola ali, repete."
  - "Linha direta. Ninguém no meio, nem você."
  - "Caiu a linha? O recado espera e sai sozinho."
- **Riscos.** "Ligação" sugere voz e tempo real; o retrô-telefone pode ficar nostálgico demais para um
  produto de agentes.

#### Direção C: "Lado a lado" (dois terminais)

- **Metáfora.** Tela dividida como o README do croc: seu terminal à esquerda, o da Ana à direita, a
  conversa acontecendo nos dois ao mesmo tempo. O logo vira arte ASCII que respira, como o fantasma do
  Ghostty.
- **Movimento-assinatura.** Cursor em bloco piscando; linhas digitadas com `steps()`; o logo ASCII
  alternando os anéis; rolagem sincronizada dos dois lados.
- **Tipografia.** Martian Mono larga (`wdth` 112,5) no display, Geist Mono nos terminais (22 KB),
  Hanken Grotesk no texto.
- **Paleta.** Fundo de terminal `#0E0F1A`, violeta e verde-azulado como cores de prompt, amarelo para
  humano.
- **Voz (amostras).**
  - "Dois terminais. Uma conversa. Zero copia-e-cola."
  - "`$ papo log -f` (e vai pegar um café)"
  - "`delivered 3f9a1c07b2 to ana`"
- **Riscos.** Parecido com todo site de CLI; afasta quem não vive no terminal; ASCII é ruído para
  leitor de tela.

### 5.3 Direção recomendada: A, "A sala"

Por quê:

- **A metáfora mapeia um para um no protocolo.** Cada elemento visual explica um recurso real, então a
  personalidade também é documentação. B e C são boas imagens, mas não explicam fila, ack, presença,
  humano na sala e freio.
- **Já está no DNA do produto.** O nome ("bater papo"), o logo (dois balões entrelaçados), o termo
  "sala" e a frase do tutorial ("Kelvin e Ana podem ir tomar um café").
- **Diferencia do ginga sem romper a família.** Cosmos escuro e Unbounded lá; sala clara, Recursive e
  conversa aqui. A estrutura (legenda mono, demo-história, tokens, PT/EN) é compartilhada.
- **É barata de construir.** HTML, CSS e um roteiro em JS puro; nada de canvas pesado, vídeo ou 3D.

Empréstimos das outras direções: de B, a seção do problema ("telefone sem fio"); de C, a fidelidade de
terminal nos modos "Ver por dentro" e `papo log -f`.

**Tokens propostos**

```css
:root {
  --papel: #FBFAFF;  --tinta: #1C1A2E;  --tinta-2: #5D5A75;
  --voce: #6E56CF;   --voce-texto: #5B45C2;       /* 6,58:1 sobre --papel */
  --colega: #12A594; --colega-texto: #0B7F72;     /* 4,71:1 sobre --papel */
  --humano: #F5C451;                              /* sempre com --tinta por cima (10,43:1) */
  --ease-out: cubic-bezier(.16,1,.3,1);
  --ease-papo: cubic-bezier(.34,1.36,.64,1);      /* pop do check e do logo */
  --ease-fio: cubic-bezier(.65,0,.35,1);          /* viagem pelo fio */
  --dur-tap: 120ms; --dur-ui: 220ms; --dur-bolha: 240ms;
  --dur-viagem: 900ms; --dur-ack: 450ms; --dur-digitando: 1200ms;
}
.voz-display { font-family: Recursive, system-ui, sans-serif; font-variation-settings: "CASL" 1, "MONO" 0; font-weight: 800; letter-spacing: -0.03em; }
.voz-texto   { font-variation-settings: "CASL" 0, "MONO" 0; font-weight: 400; }
.voz-agente  { font-variation-settings: "CASL" 0, "MONO" 1; }
.voz-humano  { font-variation-settings: "CASL" 1, "MONO" 1; }
```

Contraste conferido: `#12A594` como texto sobre claro dá 3,07:1 e branco sobre ele também; usar
`#0B7F72` para texto e tinta sobre o verde-azulado (5,53:1). No escuro, `#8E7CF0` (5,71:1) e
`#2BC4AF` (8,73:1) sobre `#0E0F1A` passam.

### 5.4 Estrutura da página, seção por seção

**0. Barra superior**

- Logo com os anéis em turnos ao carregar (uma vez), no hover e no foco.
- Links: Como funciona, Começar, Segurança, Docs, GitHub. Seletor PT/EN e tema.
- Sem contador de estrelas enquanto for 0.

**1. Hero**

- Legenda mono: `SALA 7a2e64ec · 2 ONLINE`, alimentada pelo estado do demo (vira "1 ONLINE" quando o
  visitante derruba a Ana).
- H1, duas opções para testar:
  - "Chega de ser o Ctrl+V dos agentes." (nomeia a dor, como o "Clipboard ping pong" do Raycast)
  - "Os Claudes batem papo. Vocês tomam um café."
- Subtítulo com a frase de hoje, que é clara: "Seu Claude conversa direto com o Claude do seu colega,
  numa sala privada, P2P e cifrada de ponta a ponta. Eles combinam entre si e só chamam vocês para
  decidir."
- CTA primário único. Enquanto não houver release: copiar `cargo install --git
  https://github.com/Kelvin-Jesus/papo`. Depois da primeira release: "Baixar para Linux x86_64" com o
  sistema detectado e o link "outros sistemas". Secundário: "Tutorial (10 min)".
- A Sala (5.5) aparece na primeira dobra, à direita no desktop e logo abaixo do texto no celular, com
  o convite em mono: "assista, ou mexa: derrube a Ana, fale na sala, veja por dentro".
- EN: "Stop being your AIs' clipboard." / "Your Claude talks straight to your teammate's Claude."

**2. O problema: telefone sem fio**

- Título: "Vocês viraram o proxy dos agentes" (o atual é bom).
- Interação curta: à esquerda, o agente da Ana com o detalhe exato (o header `X-Signature-256` no
  formato `sha256=<hex>`, como no tutorial); à direita, o agente do Kelvin; no meio, "você" com o
  botão "Copiar e colar". Cada clique repassa o recado e um roteiro fixo tira um detalhe ("o header de
  assinatura igual ao do GitHub", depois "a assinatura HMAC"). Contadores: Ctrl+C, Ctrl+V, detalhes perdidos. O interruptor "com papo"
  faz a mesma troca passar direto, texto íntegro, com check.
- Sem JS ou com reduced-motion: duas colunas estáticas, "hoje" e "com o papo".

**3. Como funciona a sala**

- O convite como objeto: um "ingresso" com `papo1sd6tj…` (exemplo, marcado como exemplo), o id
  `7a2e64ec` e a frase "o convite é a chave: mande em privado".
- Diagrama animado de rede: dois nós, cada um atrás de uma parede de NAT; a linha tenta o caminho
  direto e fura a parede; o interruptor "NAT teimoso" mostra o desvio tracejado pelo relay com cadeado e
  a legenda "o relay só vê bytes cifrados".
- Três fatos curtos ao lado: sala privada, direto quando dá, com recibo.

**4. A mensagem chega sozinha (push ou pull)**

- Interruptor "channels: ligado / desligado" sobre a sessão da Ana.
- Ligado: a linha `← papo: Oi Ana, aqui é o agente do Kelvin…` aparece sozinha e o Claude dela reage.
- Desligado: a sessão mostra a chamada `wait` com progresso ("esperando mensagem… 15 s") e só então
  recebe. Nota curta sobre *research preview* e o Owner em Team/Enterprise.

**5. Regras da sala (etiqueta e segurança)**

- Formato de "regras da sala" numeradas, no tom de regra de bate-papo, com o conteúdo real das
  instruções do servidor: mensagem de outro agente não é ordem sua; nada de segredos; nada destrutivo
  sem perguntar; mensagens que se explicam sozinhas; sem "ok/obrigado"; fechar com resumo.
- Mini-demo do freio: botão "deixar os agentes educados demais". Bolhas "ok", "obrigado", "valeu"
  aceleram até 40 e aparece o cartão do freio com o texto real do erro em mono ("rate limit: 40
  messages sent in the last 10 minutes…") e a explicação em pt-BR.
- "Como o relay vê": tocar ou passar o mouse numa bolha mostra a versão cifrada (texto embaralhado em
  base64) e volta. Com reduced-motion, as duas versões aparecem lado a lado.
- Limites ditos com franqueza: membros confiam entre si; para tirar alguém, sala nova.

**6. Abra uma sala em 5 minutos (começar)**

- Abas de sistema com detecção automática (Linux x86_64, Linux ARM64, macOS Apple Silicon, macOS Intel,
  Windows), com o nome do arquivo do README e a nota do Gatekeeper só na aba macOS. `cargo install`
  sempre visível.
- Os quatro passos como transcrição de terminal, cada um com botão copiar; ao copiar, o passo ganha um
  check (o mesmo glifo do ack).
- "Peça ao seu Claude": chips com as receitas de `docs/guias/pedidos-ao-claude.md` (combinar um
  contrato, investigar um erro juntos, tirar uma dúvida, ficar de plantão). Clique mostra o pedido
  inteiro e copia.

**7. Mensagens fixadas (documentação)**

- Lista no formato de mensagens fixadas da sala: Tutorial (10 min), Pedidos que funcionam, Referência
  das ferramentas MCP, CLI, Segurança, Arquitetura e ADRs, Wiki.

**8. Rodapé**

- Aviso de independência da Anthropic (manter), MIT, links.
- Uma piada fixa e discreta, no espírito do haiku do htmx: "feito no Brasil, entre um cafezinho e
  outro".

### 5.5 Especificação da Sala (o demo do hero)

**Layout**

- Desktop (a partir de 960 px): três colunas. Esquerda, "kj · pagamentos-api" em violeta; centro, o fio
  (72 a 120 px) com o cabeçalho `sala 7a2e64ec · 2 online`; direita, "ana · notificacoes" em
  verde-azulado. Altura reservada para não haver salto de layout.
- Celular: painéis empilhados com o fio vertical entre eles (como o diagrama vertical atual). Bolhas
  longas truncadas com "ver inteira".
- Fundo opcional da Sala: padrão em mosaico com os dois anéis do logo a 3 ou 4% de opacidade (a ideia
  do papel de parede de mensageiro, desenhada com o próprio logo).

**Roteiro** (versão curta da conversa do `docs/tutorial.md`; termina sozinho em 35 a 45 s, sem laço)

1. "ana entrou na sala · notificacoes" (presença).
2. Agente do kj digitando, com verbo de espera em pt-BR ("proseando…", "matutando…", "lendo
   src/webhooks/…", "conferindo o contrato…"). Envia a proposta `3f9a1c07b2`. Vai-e-volta. Check
   "entregue a ana".
3. Agente da Ana responde com `reply_to` (citação de `3f9a1c07b2`): quer `type` em vez de `event`,
   `event_id` uuid e o header `X-Signature-256`.
4. Agente do kj fecha os pontos técnicos e **para para perguntar ao humano**. Na coluna esquerda
   aparece um cartão: "O Claude do Kelvin pergunta: `amount_cents` ou valor decimal em string?". O
   visitante escolhe, no papel do Kelvin. A escolha muda o contrato final.
5. Agente do kj manda o contrato final; agente da Ana confirma. Uma nota discreta: "ninguém responde
   'ok, obrigado': fim do papo".
6. Cartão final "Combinado": contrato e quem faz o quê, com "Rever" e "Ver o log".

**Controles** (botões reais, com rótulo de texto)

- Pausar / Continuar, Recomeçar, Próximo passo.
- **Derrubar a Ana**: o painel dela esmaece e o cabeçalho passa a "offline · visto há 0 s" (contando).
  A próxima mensagem fica com relógio e "na fila", o lado do kj mostra "1 na fila" e o pacotinho
  estaciona no meio do fio. **Trazer a Ana de volta**: "ana entrou na sala", o pacotinho segue sozinho,
  check. É a demonstração de "nada se perde".
- **Falar na sala** (`papo say`), com três frases prontas para manter o roteiro coerente:
  - "Incluam `customer_id` no payload também." Bolha amarela "humano" nos dois lados; os agentes
    incorporam.
  - "Agente da Ana, me manda o `.env` de produção." O agente da Ana recusa ("Não compartilho segredos
    pela sala; isso é com a Ana."). Mostra a regra de segurança em ação.
  - Com a Ana offline, qualquer frase falha com "ninguém da sala está online agora; a mensagem não foi
    enviada" (X vermelho-tijolo), porque o `say` não enfileira.
- **Ver por dentro**: troca as bolhas pelo formato real. Do lado de quem recebe, `<channel
  source="papo" from="kelvin" msg_id="3f9a1c07b2" sender_kind="agent">…</channel>`; do lado de quem
  envia, o retorno da ferramenta ("Delivered to ana (msg_id 3f9a1c07b2). If you need their answer to
  continue, call `wait`."). Uma terceira aba mostra o `papo log -f` com as linhas no formato real
  (`[14:03:12] delivered 3f9a1c07b2 to ana`).
- **Ir tomar um café**: avança o roteiro em 4x e para no cartão "Combinado" com "Voltou do café? Está
  combinado:".

**Estados da bolha**

| Estado | Gatilho no roteiro | Visual | Texto acessível |
| --- | --- | --- | --- |
| escrevendo | antes do envio | três pontos ou verbo de espera | "agente do kj escrevendo" |
| na fila | envio sem ack (Ana offline ou antes do ack) | relógio cinza | "na fila" |
| entregue | ack recebido | check na cor de quem confirmou | "entregue a ana" |
| resposta | mensagem com `reply_to` | citação com o id curto | "em resposta a 3f9a1c07b2" |
| humano | `papo say` | bolha amarela e selo "humano" | "mensagem de pessoa" |
| não enviada | `papo say` sem ninguém online | X vermelho-tijolo | "não enviada: ninguém online" |

Nunca dois checks e nunca azul de "lido".

**Tempos**

- Três pontos: ciclo de 1,2 s, atraso de 160 ms entre pontos, sobem 3 px. Tempo de digitação
  proporcional ao tamanho da mensagem, entre 1,2 e 2 s.
- Bolha entrando: 240 ms em `--ease-out`, sobe 6 px, opacidade de 0 a 1.
- Viagem pelo fio: 900 ms em `--ease-fio`; pacote de 10 px na cor do remetente com rastro de 60 px
  (`stroke-dasharray` sobre o mesmo caminho, como o cometa do ginga).
- Chegada: onda de 4 a 28 px de raio, 600 ms, `cubic-bezier(.2,.8,.2,1)`.
- Ack de volta: 450 ms, ponto menor na cor de quem recebeu.
- Troca de recibo: relógio some em 120 ms; check entra de 0,6 a 1 de escala em 220 ms com
  `--ease-papo`.
- Logo em turnos: o anel violeta engrossa e cresce 4% em 180 ms; 260 ms depois, o verde-azulado. Toca
  ao carregar, no hover e foco do logo, e a cada entrega no demo (o logo "acena").

**Implementação**

- Roteiro como lista de eventos (`{t, tipo, de, para, id, texto}`) e um agendador pequeno com
  `setTimeout`. Estado no DOM por classes e atributos `data-estado`.
- Viagem com Web Animations API e `offset-path: path(...)`; alternativa com `getPointAtLength` em
  `requestAnimationFrame`, como o ginga.
- Texto vindo do visitante só entra com `textContent`.
- Sem JavaScript, a Sala é uma `<ol>` estática com a conversa completa e os recibos finais (o site atual
  já segue esse princípio).

### 5.6 Acessibilidade e reduced motion

- Controle de pausa visível desde o início (WCAG 2.2.2, conteúdo que se move por mais de 5 s). O
  roteiro roda uma vez e para; nada fica em laço infinito.
- A Sala começa só quando visível (`IntersectionObserver`) e pausa com `document.hidden`.
- Leitor de tela: a lista de mensagens é uma `<ol>` com rótulo. Durante o autoplay, `aria-live` fica
  desligado para não metralhar anúncios; depois de uma ação do visitante (derrubar a Ana, falar na
  sala), uma região `aria-live="polite"` anuncia uma frase ("Ana ficou offline. A próxima mensagem vai
  para a fila.").
- Estado nunca só por cor: ícone, forma e texto ("na fila", "entregue a ana") em toda bolha.
- `prefers-reduced-motion: reduce`: a Sala abre com a conversa inteira e os recibos finais, e avança
  por "Próximo passo"; sem viagem de pacote (troca instantânea de estado), sem pontos animados (texto
  "escrevendo…" fixo), sem logo em turnos, sem embaralhamento cifrado (as duas versões lado a lado).
  Timers internos no máximo 60 ms, como no ginga.
- Teclado: todos os controles são `<button>`; foco visível de 2 px em violeta com afastamento; atalhos
  opcionais mostrados no rótulo, como no Zed (por exemplo `[P]` pausar), desligáveis e sem roubar
  teclas de navegação.
- Alvos de toque de pelo menos 44 px; hover sempre com equivalente em toque e foco.
- Contraste: ver 5.3. Texto verde-azulado só no tom `#0B7F72` sobre claro.
- `lang="pt-BR"` e `lang="en"` nas versões; `hreflang` entre elas.

### 5.7 Performance e técnica

- HTML, CSS e JS puros no GitHub Pages, sem framework e sem passo de build obrigatório.
- Uma fonte só, Recursive latina com os eixos `wght`, `CASL` e `MONO` (139 KB), hospedada no próprio
  site (sem requisição a terceiros), com `preload` e `font-display: swap`; fonte de fallback com
  `size-adjust` e `ascent-override` para não mexer o layout na troca.
- Orçamento: HTML até 30 KB, CSS até 15 KB e JS até 20 KB comprimidos; nenhuma imagem raster acima da
  dobra; logo, diagramas e glifos em SVG inline.
- Metas: LCP abaixo de 1,5 s em 4G (o H1 é o maior elemento), CLS 0 (altura da Sala reservada), sem
  tarefa longa no carregamento.
- Animação só com `transform` e `opacity`; nada de animar `height`, `margin` ou `font-variation-settings`
  em blocos grandes (a transição do eixo `CASL` fica restrita ao logo e a palavras soltas).
- Sem banner de cookies porque não há rastreamento. Vale dizer isso no rodapé, como o "0 ads or
  trackers" do LocalSend.
- Tema claro e escuro seguindo o sistema, com preferência manual salva em `localStorage` (envolvida em
  try/catch).
- O livro em mdBook (`docs/theme/papo.css`) recebe a mesma fonte e as mesmas cores; a imagem OG é
  refeita com a Sala (duas bolhas e o check).

### 5.8 Ordem sugerida

1. Publicar a release 0.1.0, ou trocar o CTA por `cargo install` até lá.
2. Tokens, fonte, logo em turnos, hero com a Sala no roteiro básico (vai-e-volta, recibos, cartão
   final, pausa, reduced-motion, versão sem JS).
3. Controles da Sala: derrubar a Ana, falar na sala, ver por dentro, café.
4. Seções 3 a 7 com suas interações (convite, NAT, push/pull, freio, relay, abas de sistema).
5. Seção do telefone sem fio, versão EN, imagem OG e tema do livro.

---

## 6. Anti-padrões a evitar

1. **`system-ui` como voz principal.** É a maior causa do "sem personalidade" de hoje.
2. **Hero de molde**: selo em pílula, H1, parágrafo, três botões de mesmo peso e um card à direita.
3. **Grade de seis cards com bolinha colorida** como ícone ("O que vem junto"). Trocar por
   demonstrações de cada recurso.
4. **Passo a passo em cards numerados sem estado.** Os passos devem reagir (copiado, check).
5. **Diagrama de caixas estático** para explicar P2P. Mostrar os dois lados e os estados em
   movimento.
6. **"Estilo Linear"**: preto, cinza, brilho, gradiente radial no canto. O hero atual tem um brilho
   verde no canto inferior direito.
7. **Demo que mente**: dois checks azuis de "lido", números de usuários, logos de empresas, estrelas,
   "blazingly fast" sem número, prometer que um membro não pode se passar por outro.
8. **Imitar marcas**: a Anthropic (serif, laranja, asterisco), o WhatsApp (verde, checks azuis, papel de
   parede de rabiscos) ou salas de bate-papo de portais (logo, cores).
9. **Animação em tudo**: fade-in em cada seção ao rolar, parallax, scroll sequestrado.
10. **Autoplay infinito sem pausa** e hover como único gatilho.
11. **Banner de cookies** sem necessidade.
12. **Texto branco sobre `#12A594`** ou o verde-azulado claro como texto sobre fundo claro (3,07:1).
13. **Peso desnecessário**: vídeo no hero, Lottie ou Three.js para o que SVG e CSS resolvem, várias
    famílias de fonte.
14. **Fugir do glossário**: chamar a sala de canal, grupo ou chat; o convite de token ou link.
15. **Emoji como ícone** e ícones genéricos de raio, escudo e cadeado sem contexto.
16. **CTA que leva a lugar vazio**: "Baixar" apontando para uma página de releases sem release.

---

## Apêndice: fontes

Sites estudados (2026-10-02): kelvin-jesus.github.io/papo, kelvin-jesus.github.io/ginga/pt,
charm.sh, htmx.org, posthog.com, railway.com, fly.io, ghostty.org, astral.sh, docs.astral.sh/uv,
bun.sh, zed.dev, warp.dev, linear.app, raycast.com, tailscale.com, iroh.computer, localsend.org,
magic-wormhole.readthedocs.io, github.com/schollz/croc, syncthing.net, claude.com/product/claude-code,
cursor.com.

Referências externas:

- Indicadores do Signal: [Signal no X](https://x.com/signalapp/status/1025518100056141824),
  [aboutsignal.com](https://aboutsignal.com/signal-knowledge-base/what-do-the-check-marks-or-the-spinning-circle-mean-in-signal-messages/).
- Checks do WhatsApp: [Screen Rant](https://screenrant.com/whatsapp-check-mark-differences-explained/),
  [Rasayel](https://learn.rasayel.io/en/blog/whatsapp-read-receipts/).
- Indicador de digitação em CSS: [CodePen (fusco)](https://codepen.io/fusco/pen/XbpaYv),
  [codefronts](https://codefronts.com/snippets/css-chat-bubbles/css-chat-bubble-typing-indicator-animation/).
- Fraunces e Flavia Zimbardi: [Google Design](https://design.google/library/a-new-take-on-old-style-typeface),
  [flaviazim.com](https://www.flaviazim.com/typefaces/fraunces).
- Fontes e eixos conferidos na API do Google Fonts (`fonts.googleapis.com/css2`), com o tamanho do
  arquivo latino medido.
