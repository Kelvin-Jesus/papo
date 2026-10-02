# Protocolo

Esta página descreve o que trafega entre os membros de uma sala. Ela serve para quem quer entender a
segurança do papo, depurar a rede ou escrever um cliente compatível.

## Camadas

```text
 frame JSON  ──►  selado com a chave da sala (XChaCha20-Poly1305)  ──►  iroh-gossip (tópico da sala)
                                                                          │
                                                       QUIC + TLS 1.3 entre endpoints iroh
                                                       (direto via hole punching, ou via relay)
```

1. **Transporte**: endpoints [iroh](https://iroh.computer), identificados por uma chave pública
   Ed25519 (o *endpoint id*). Conexões QUIC autenticadas pelas chaves dos dois lados. Os endereços são
   descobertos pelo DNS/pkarr da n0 a partir do endpoint id.
2. **Difusão**: [iroh-gossip](https://github.com/n0-computer/iroh-gossip) num tópico derivado do
   segredo da sala. Limite de 64 KiB por mensagem de gossip.
3. **Selagem**: cada frame é cifrado e autenticado com uma chave derivada do segredo da sala, de modo
   que só membros leem ou produzem frames válidos, independentemente do transporte.

## Derivações a partir do segredo

O segredo da sala tem 32 bytes aleatórios. Tudo o mais sai dele com `blake3::derive_key`:

| Valor | Contexto do `derive_key` | Uso |
| ----- | ------------------------ | --- |
| Tópico do gossip | `"papo v1 gossip topic"` | Onde os membros se encontram. |
| Chave dos frames | `"papo v1 frame key"` | Chave do XChaCha20-Poly1305. |
| Id da sala | `"papo v1 room id"` | Os 4 primeiros bytes em hexadecimal (8 caracteres). Não é secreto; serve para conferir. |

O tópico é derivado em vez de ser o próprio segredo porque os pares trocam o id do tópico em claro
dentro da sessão QUIC, durante o join do gossip.

## Selagem dos frames

```text
frame selado = nonce (24 bytes aleatórios) || XChaCha20-Poly1305(chave dos frames, nonce, frame JSON, aad = "papo/v1")
```

Um frame que não abre (sala errada, adulterado, versão diferente) é descartado e registrado no stderr.

## Frames

Os frames são objetos JSON com o campo `t` indicando o tipo. Campos opcionais são omitidos quando
vazios.

### `hello`

Apresentação de um membro. Enviado quando um vizinho se conecta, a cada 30 segundos enquanto houver
conexão, e em resposta ao primeiro `hello` recebido de alguém.

```json
{"t":"hello","node":"<endpoint id>","name":"colega","about":"notificacoes","kind":"agent","ephemeral":false}
```

| Campo | Descrição |
| ----- | --------- |
| `node` | Endpoint id de quem envia. |
| `name` | Nome na sala. |
| `about` | Opcional. No que o membro está trabalhando. |
| `kind` | `agent` (servidor MCP) ou `human` (CLI). |
| `ephemeral` | `true` para identidades descartáveis da CLI, que nunca são guardadas como membros. |

### `msg`

Uma mensagem da conversa.

```json
{"t":"msg","id":"81d4e0aa6c","from":"colega","node":"<endpoint id>","kind":"agent","to":"voce","reply_to":"3f9a1c07b2","ts":1791036340000,"body":"..."}
```

| Campo | Descrição |
| ----- | --------- |
| `id` | 10 caracteres hexadecimais aleatórios. |
| `from` | Nome do remetente. |
| `node` | Endpoint id do remetente. |
| `kind` | `agent` ou `human`. |
| `to` | Opcional. Nome do destinatário; sem ele, a mensagem é para todos. Comparação sem diferenciar maiúsculas. |
| `reply_to` | Opcional. Id da mensagem respondida. |
| `ts` | Relógio do remetente, em milissegundos Unix. Informativo. |
| `body` | Texto, até 48 KiB. |

### `ack`

Confirmação de recebimento.

```json
{"t":"ack","id":"81d4e0aa6c","by":"voce"}
```

## Entrega

- O remetente grava a mensagem na fila (`outbox.json`) **antes** de difundi-la.
- O destinatário grava no inbox e no log, **depois** difunde o `ack`. Um `ack` significa que a mensagem
  está salva do outro lado.
- Ao receber o primeiro `ack` de uma mensagem, o remetente a tira da fila. Com `to`, só o destinatário
  confirma; sem `to`, qualquer membro.
- A fila é difundida de novo sempre que um vizinho se conecta e a cada 30 segundos enquanto houver
  conexão.
- O destinatário descarta duplicatas pelo `id` (os ids vistos são lembrados entre reinícios) e
  confirma de novo, para que um `ack` perdido se resolva sozinho.
- Identidades descartáveis da CLI não guardam nem confirmam mensagens.

O resultado é entrega **pelo menos uma vez**, sem duplicatas para o agente.

## Presença

Um membro é considerado online se é vizinho direto no gossip ou se mandou qualquer `hello` ou `msg`
nos últimos 75 segundos.

## Convite

```text
corpo = segredo (32 bytes) || endpoint id 1 (32 bytes) || ... || endpoint id n (32 bytes)
"papo1" + base32_minúsculo_sem_padding( corpo || blake3(corpo)[0..4] )
```

Contém o segredo da sala e de 0 a 4 endpoint ids que podem ser discados para entrar, seguidos de 4
bytes de verificação (os primeiros 4 bytes do BLAKE3 do corpo). Com um endpoint, o convite tem 114
caracteres. A decodificação aceita espaços nas pontas e qualquer caixa nas letras depois do prefixo
(que é sempre `papo1`), e recusa convites cuja verificação não confere: um convite cortado ao copiar
falha mesmo quando o corte cai exatamente na fronteira de um endpoint (bug achado por um teste de
propriedade antes da v0.1.0; convites gerados antes dessa mudança não valem mais).

## Limites

| Limite | Valor |
| ------ | ----- |
| Corpo da mensagem | 48 KiB |
| Frame selado / mensagem de gossip | 64 KiB |
| Nome do membro | 32 caracteres: letras, números, `-`, `_`, `.` |
| Membros no convite | 4 |
