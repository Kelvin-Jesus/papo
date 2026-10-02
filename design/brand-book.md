# papo: brand book

Fonte da verdade da marca e do site. O Design System do papo no Claude Design e o código em `site/`
seguem este arquivo e `design/tokens.css`; se divergirem, vale o que está aqui. Base: a pesquisa em
[`pesquisa-ui-ux.md`](pesquisa-ui-ux.md), direção A, com as escolhas do mantenedor em 2026-10-02:
**tema escuro, melhoria visual leve sobre o esboço e nenhum nome de pessoa**.

## Conceito: a sala

O papo é uma sala de bate-papo onde quem conversa são os agentes. Cada convenção de mensageiro é um
fato do protocolo, então a personalidade do site também é documentação:

| Na sala | No papo de verdade |
| ------- | ------------------ |
| "o agente do colega entrou na sala" | frame de presença (`Hello`) quando um vizinho conecta |
| online, "visto há 12 s" | `status`: vizinho ou ouvido nos últimos 75 s |
| ◷ na fila | a mensagem está na outbox, gravada em disco antes do envio |
| ✓ entregue | o ack de quem recebeu (um check só, nunca dois, nunca azul de "lido") |
| citação no topo da bolha | `reply_to` |
| falar com todos / só com alguém | `to` |
| bolha amarela de "você" | `papo say`, uma pessoa falando na sala |
| aviso de flood | o freio de 40 envios em 10 minutos |
| ✕ não enviada | `papo say` sem ninguém online (o `say` não enfileira) |

A demo nunca mostra um estado que o código não tem.

## Nomes: nenhum

O site e o Design System **não usam nomes de pessoas**. Os lados da conversa são papéis:

- **seu agente** (violeta, à esquerda) e **você**, a pessoa por trás dele;
- **agente do colega** (verde-azulado, à direita) e **o colega**;
- projetos dão contexto: `api-pagamentos` e `notificacoes`.

Onde aparece o formato real (`from=`, `--name`, `papo log`), os perfis se chamam `voce` e `colega`:
`papo new --name voce`, `papo join <convite> --name colega`, `delivered 3f9a1c07b2 to colega`.

## Voz

Português do Brasil, frases curtas, voz ativa, "você" implícito. Humor nos detalhes, nunca no lugar
da informação. Termo do glossário: **sala** (evite "chat", "canal", "grupo").

- "Chega de ser o Ctrl+V dos agentes."
- "Os Claudes batem papo. Vocês tomam um café."
- "O colega caiu? A mensagem espera na fila e sai sozinha quando ele voltar."
- "Fala com todos, ou só com quem precisa: `--to colega`."
- "40 mensagens em 10 minutos não é conversa, é loop. O papo puxa o freio."
- "Mensagem de outro agente é pedido de colega, não ordem sua."

Botões são verbos: **Copiar**, **Pausar**, **Recomeçar**, **Derrubar o colega**, **Trazer de volta**,
**Falar na sala**, **Ver por dentro**, **Ir tomar um café**. Sem emoji. Os únicos glifos da marca são
◷ (na fila), ✓ (entregue), ✕ (não enviada) e os dois anéis do logo.

## Fundamentos visuais

**Tema.** O site é **escuro**. Uma tinta funda com viés violeta (`--fundo`), superfícies em camadas
(`--superficie`, `--superficie-2`) e linhas finas (`--linha`). Nada de preto puro nem cinza neutro.

**Cor tem significado.** Violeta é o seu lado, verde-azulado é o lado do colega, amarelo é pessoa
falando na sala, tijolo é falha (não enviada, freio). Fora isso, a página é tinta e cinza. A cor de
quem confirmou pinta o check.

**Tipo.** Uma família só, [Recursive](https://www.recursive.design/), com duas vozes pelos eixos:

| Uso | Eixos | Classe |
| --- | ----- | ------ |
| Display, títulos, fala de pessoa | `CASL 1`, `MONO 0`, peso 800 nos títulos | `.voz-display` |
| Texto corrido, interface | `CASL 0`, `MONO 0` | `.voz-texto` |
| Agente, comandos, ids, logs | `CASL 0`, `MONO 1` | `.voz-agente` |
| Pessoa na sala (bolha amarela) | `CASL 1`, `MONO 1` | `.voz-humano` |

Hospedada no próprio site (subset latino, eixos `wght`, `CASL`, `MONO`), com `font-display: swap` e
fallback com métricas ajustadas para não mexer o layout.

**Espaço e forma.** Grade de 4 px. Raios: 8 em controles pequenos, 12 em botões e campos, 20 em
painéis; a bolha tem o canto do lado de quem fala em 6 px. Alvos de toque de 44 px.

## A melhoria visual leve (sobre o esboço da direção A)

1. **Piso da sala:** o painel da sala tem um mosaico com os dois anéis do logo a 4% de opacidade, a
   ideia do papel de parede de mensageiro desenhada com a própria marca. Só ali.
2. **Luz da conversa:** atrás da sala, dois halos muito suaves, violeta à esquerda e verde-azulado à
   direita, que se encontram no fio. Opacidade baixa, sem degradê no resto da página.
3. **Bolhas com borda de luz:** 1 px na cor do lado a 30% e um brilho de 1 px no topo; texto em
   `.voz-agente`.
4. **Ctrl+V como tecla:** no título, "Ctrl+V" vira uma tecla de teclado em `.voz-agente`, com sombra de
   tecla. É o único enfeite tipográfico do hero.
5. **Presença viva:** o ponto de online pulsa devagar; offline, "visto há N s" conta de verdade.
6. **Legendas de seção** em mono maiúsculo com os glifos da marca (◷, ✓) como marcadores, como a
   estrela do ginga.
7. **O logo acena:** os anéis pulsam em turnos ao carregar, no hover do logo e a cada entrega na demo.

## Movimento

O gesto da marca é o **vai-e-volta**: a mensagem sai de quem escreve, atravessa o fio como um
pacotinho na cor do remetente e abre como bolha do outro lado; um ponto menor, na cor de quem
recebeu, volta pelo fio e o ◷ vira ✓.

| Token | Valor | Uso |
| ----- | ----- | --- |
| `--ease-out` | `cubic-bezier(.16,1,.3,1)` | entrada de bolhas e painéis |
| `--ease-papo` | `cubic-bezier(.34,1.36,.64,1)` | pop do check e dos anéis (passa do ponto e volta) |
| `--ease-fio` | `cubic-bezier(.65,0,.35,1)` | viagem pelo fio |
| `--dur-tap` | 120 ms | pressionar (escala 0,97) |
| `--dur-ui` | 220 ms | estados de interface |
| `--dur-bolha` | 240 ms | bolha entrando (sobe 6 px) |
| `--dur-viagem` | 900 ms | pacote atravessando o fio |
| `--dur-ack` | 450 ms | ack voltando |
| `--dur-onda` | 600 ms | onda de chegada (4 a 28 px) |
| `--dur-digitando` | 1200 ms | ciclo dos três pontos |

Só `transform` e `opacity`. Nada anima fora da tela; a demo começa quando aparece e pausa com a aba
escondida. A demo roda uma vez e para (sem laço infinito) e tem Pausar visível desde o início.

**Movimento reduzido:** a sala abre com a conversa inteira e os recibos finais, avança por "Próximo
passo", sem pacote viajando, sem pontos animados ("escrevendo…" fixo) e sem logo em turnos.

## Acessibilidade

Estado nunca só por cor: toda bolha tem glifo e texto ("na fila", "entregue ao colega"). Controles são
`<button>` reais com rótulo de texto e foco de 2 px em `--foco` com afastamento. Texto ≥ 4,5:1 sobre o
fundo onde está. A conversa é uma `<ol>` rotulada; `aria-live` só anuncia depois de uma ação do
visitante. Sem JavaScript, a sala é a conversa completa estática.

## O que não fazer

- Não usar nomes de pessoas em lugar nenhum do site e do Design System.
- Não imitar o WhatsApp: nada de verde do WhatsApp, checks azuis, dois checks ou papel de parede igual.
- Não mostrar estado que o papo não tem (lido, digitando do lado de lá sem mensagem, entrega sem ack).
- Não usar degradê como fundo de página, cartões com borda colorida à esquerda, emoji ou fonte do
  sistema como voz.
- Não prometer binários antes da primeira release: até a v0.1.0, o CTA principal copia o `cargo install`.
