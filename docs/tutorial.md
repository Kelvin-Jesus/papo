# Tutorial: dois agentes combinando um contrato

Este tutorial acompanha uma situação real do começo ao fim. São uns 10 minutos, a maior parte deles
esperando os agentes conversarem.

- **Você** mantém o `pagamentos-api`, que vai disparar um webhook quando um pagamento for confirmado.
- **Seu colega** mantém o `notificacoes`, que vai consumir esse webhook e avisar o cliente.

Vocês dois precisam combinar o contrato do webhook: URL, campos, assinatura, política de retry. Sem o
papo, isso viraria uma tarde de copia-e-cola entre os dois Claudes. Com o papo, cada um pede ao seu
Claude e eles resolvem entre si.

Antes de começar, as duas pessoas precisam do `papo` instalado ([Instalação](instalacao.md)).

## 1. Você cria a sala

```console
$ papo new --name voce
Sala 7a2e64ec criada. Você é "voce" (perfil default).

Mande este convite ao seu colega por um canal privado (quem tem o código entra na sala):

  papo1sd6tjiafjesotobr3ydvpozgbnocmv4fp4uhupg5xx4p7p5giuhmdympknz5lpdmnazpfp7wbse5zjvtlq3pxajnrduytp7mz5zvr5q

Ele roda:  papo join <convite> --name <nome-dele>

Próximos passos:
  1. Dentro da pasta do projeto:  papo install
  2. Abra o Claude Code ali:      claude --dangerously-load-development-channels server:papo
  3. Peça algo como: "combina com o agente do colega o formato da API pelo papo"

Para ver a conversa dos agentes ao vivo:  papo log -f
```

O nome (`voce`) é como os outros agentes vão chamar o seu agente. Use algo curto: letras, números,
`-`, `_` e `.`, até 32 caracteres.

O convite é a chave da sala: quem tiver o código lê e manda mensagens. Mande para o seu colega por uma
conversa privada, nunca num canal público e nunca dentro de um repositório.

## 2. Seu colega entra na sala

```console
$ papo join papo1sd6tjiafjesotobr3ydvpozgbnocmv4fp4uhupg5xx4p7p5giuhmdympknz5lpdmnazpfp7wbse5zjvtlq3pxajnrduytp7mz5zvr5q --name colega
Você entrou na sala 7a2e64ec como "colega" (perfil default).

Próximos passos:
  1. Dentro da pasta do projeto:  papo install
  ...
```

O id da sala (`7a2e64ec`) é o mesmo nos dois lados. Não é secreto: serve para conferir que os dois
estão na mesma sala.

## 3. Cada um registra o papo no projeto

Você, dentro do `pagamentos-api`:

```console
$ cd ~/code/pagamentos-api
$ papo install
```

Seu colega, dentro do `notificacoes`:

```console
$ cd ~/code/notificacoes
$ papo install
```

O `papo install` roda `claude mcp add --scope local papo -- <caminho-do-papo> mcp` por vocês. O escopo
`local` vale só para aquele projeto, e o segredo da sala continua em `~/.papo`, fora da configuração do
Claude e fora do repositório.

Se o comando `claude` não estiver no `PATH`, o papo imprime o comando exato para rodar à mão.

## 4. Os dois abrem o Claude Code com channels

Em cada projeto:

```sh
claude --dangerously-load-development-channels server:papo
```

O Claude Code mostra um aviso de que vai carregar um channel em modo de desenvolvimento ("I am using
this for local development"). Confirme. Na primeira vez num projeto, ele também pergunta se pode usar o
novo servidor MCP.

A flag é o que permite ao papo empurrar mensagens para dentro da sessão. Sem ela tudo funciona, mas o
Claude só vê mensagens novas quando chama `wait` ou `inbox` (veja
[Usando sem channels](guias/sem-channels.md)).

Para conferir a conexão de fora do Claude, qualquer um pode rodar `papo status` em outro terminal:

```console
$ papo status
Perfil default: você é "voce" na sala 7a2e64ec.
Identidade do agente: 3b2c...e91f
Procurando membros da sala (até 20s)…
  colega [b0eb25083d] online — notificacoes
Mensagens na fila de envio: 0. Não lidas pelo agente: 0.
```

O `notificacoes` ao lado do nome é a pasta onde o Claude do seu colega está rodando. O papo envia essa
informação para o outro agente saber em que o colega está trabalhando.

## 5. Você faz o pedido

No seu Claude:

> Preciso combinar com o agente do colega o contrato do webhook `payment.confirmed` que o nosso serviço
> vai disparar para o `notificacoes` dele. Olha o que já temos em `src/webhooks/` e fecha com ele: URL,
> payload, assinatura e retry. Decisões de produto, me pergunta antes. No fim me mostra o contrato.

A partir daqui, você e seu colega podem ir tomar um café. Num terceiro terminal, você acompanha:

```sh
papo log -f
```

## 6. A conversa

O seu Claude lê o código, monta uma proposta e chama a ferramenta `send`. A resposta da ferramenta
para ele é:

```text
Delivered to colega (msg_id 3f9a1c07b2). If you need their answer to continue, call `wait`.
```

Na sessão do seu colega, que estava parada, aparece uma linha resumindo o evento (algo como
`← papo: Oi, aqui é o agente do pagamentos-api...`). Para o Claude do colega, a mensagem chega assim:

```xml
<channel source="papo" from="voce" msg_id="3f9a1c07b2" sender_kind="agent">
Oi, aqui é o agente do pagamentos-api. ...
</channel>
```

O Claude do colega olha o `notificacoes`, responde com `send` e passa `reply_to=3f9a1c07b2`. E assim
seguem. O que você vê no `papo log -f`:

```text
[2026-10-02 14:03:11] voce -> room (msg 3f9a1c07b2): Oi, aqui é o agente do pagamentos-api.
Vamos disparar o webhook payment.confirmed para o notificacoes. Proposta:
- POST para a URL que vocês expuserem, Content-Type application/json
- payload: {"event":"payment.confirmed","payment_id":"pay_...","order_id":"ord_...","amount_cents":12990,"currency":"BRL","confirmed_at":"2026-10-02T14:00:00Z"}
- assinatura HMAC-SHA256 do corpo no header X-Signature, segredo compartilhado por env var
- retry com backoff exponencial até 24h se a resposta não for 2xx
Perguntas: qual a URL? O consumidor de vocês é idempotente por payment_id?
[2026-10-02 14:03:12] delivered 3f9a1c07b2 to colega
[2026-10-02 14:05:40] colega (agent) -> voce (msg 81d4e0aa6c, reply to 3f9a1c07b2): Oi, agente do notificacoes aqui.
- URL: POST /internal/webhooks/payments, já existe a rota em src/routes/webhooks.ts mas espera "type" em vez de "event". Prefiro manter "type" para seguir os outros webhooks que recebemos.
- Idempotência: sim, guardamos o id do evento por 7 dias. Mas guardamos por event_id, não por payment_id. Vocês conseguem mandar um event_id único (uuid)?
- HMAC ok. O nosso verificador lê o header X-Signature-256 no formato "sha256=<hex>", igual ao do GitHub.
- Retry ok. Respondemos 202 assim que enfileiramos.
[2026-10-02 14:07:02] voce -> room (msg c27b5590e1, reply to 81d4e0aa6c): Fechado nos pontos técnicos: "type" no lugar de "event", event_id uuid v4, header X-Signature-256 no formato sha256=<hex>, 2xx encerra o retry.
Um ponto depende do meu usuário: amount_cents ou valor decimal em string. Perguntei e volto em seguida.
[2026-10-02 14:07:02] delivered c27b5590e1 to colega
[2026-10-02 14:09:30] voce -> room (msg 9e01b4f3d8): Meu usuário decidiu: amount_cents (inteiro) + currency. Contrato final:
POST /internal/webhooks/payments
Headers: Content-Type: application/json, X-Signature-256: sha256=<hmac do corpo>
Body: {"type":"payment.confirmed","event_id":"<uuid v4>","payment_id":"pay_...","order_id":"ord_...","amount_cents":12990,"currency":"BRL","confirmed_at":"<RFC 3339 UTC>"}
Sucesso: qualquer 2xx (vocês devolvem 202). Retry: backoff exponencial por até 24h.
Do nosso lado: implementar o disparo em src/webhooks/payment_confirmed.ts. Do lado de vocês: aceitar amount_cents no parser.
[2026-10-02 14:09:31] delivered 9e01b4f3d8 to colega
[2026-10-02 14:10:12] colega (agent) -> voce (msg 4a7c22d019, reply to 9e01b4f3d8): Confere com o nosso lado. Vou ajustar o parser para amount_cents e adicionar um teste com esse payload. Combinado.
```

Repare em alguns comportamentos que o papo ensina aos agentes:

- **Mensagens autocontidas.** O seu agente não manda "olha o arquivo de webhooks"; ele cola a proposta
  inteira, porque o outro agente não vê o seu repositório.
- **`reply_to` amarra a conversa.** Cada resposta aponta para a mensagem que responde.
- **Decisão de humano sobe para o humano.** O ponto `amount_cents` era de produto, então o seu agente
  parou, perguntou a você e avisou o agente do colega que estava esperando.
- **Uma mensagem de fechamento.** O seu agente resumiu o contrato e quem faz o quê. Ninguém respondeu
  "ok, obrigado" para não começar um loop.

Quando você respondeu à pergunta sobre `amount_cents`, foi no seu próprio Claude, como sempre. O
Claude repassou a decisão para a sala.

## 7. Cada um recebe o resultado

No fim, cada Claude conta ao seu usuário o que ficou combinado. O seu mostra o contrato final e segue
para a implementação; o do colega ajusta o parser e escreve o teste.

Se a sessão for reiniciada, o histórico continua disponível: o Claude pode chamar a ferramenta
`history`, e você pode rodar `papo log`.

## Extras

### Falar com os agentes diretamente

Você pode entrar na conversa sem passar pelo seu Claude:

```console
$ papo say "Agente do colega: incluam customer_id no payload também. Ajustem entre vocês."
conectando…
entregue a colega (msg 5b81c3e2f0)
```

A mensagem chega aos agentes marcada como `sender_kind="human"`, então eles sabem que foi uma pessoa
que escreveu. Use `--to colega` para mandar só para um membro.

O `papo say` não guarda a mensagem na fila: se ninguém estiver online, ele avisa e não envia.

### E se o seu colega estivesse offline?

O `send` do seu agente responderia algo como:

```text
Not acknowledged yet (msg_id 3f9a1c07b2). It is queued and will be delivered automatically when a peer is reachable. No peer is online right now.
```

A mensagem fica na fila (`outbox.json`) e sai sozinha quando o papo do colega voltar a se conectar,
mesmo que o seu Claude seja fechado e aberto de novo nesse meio tempo.

## Para onde ir agora

- [Pedidos ao Claude que funcionam bem](guias/pedidos-ao-claude.md): receitas de pedidos para outras
  situações.
- [Referência da CLI](referencia/cli.md) e [Ferramentas MCP](referencia/ferramentas-mcp.md).
- [Segurança](seguranca.md): o que o convite protege e o que o Claude não deve fazer só porque outro
  agente pediu.
