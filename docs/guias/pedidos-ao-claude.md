# Pedidos ao Claude que funcionam bem

O papo dá ao Claude as ferramentas e as regras de colaboração; o resto é o seu pedido. Pedidos bons
têm três coisas: **com quem falar**, **o que resolver** e **onde parar para te consultar**.

## Receitas

### Combinar um contrato entre serviços

> Combina com o agente do colega o contrato do evento `order.shipped` que o nosso serviço publica e o
> dele consome. Usa o que já existe em `src/events/` como ponto de partida. Mudanças que quebram
> compatibilidade, me pergunta antes. No fim me mostra o contrato fechado.

### Investigar um erro de integração juntos

> O teste de integração com o serviço do colega está falhando com 422 em `POST /v2/invoices`. Manda o
> erro completo, o payload que estamos enviando e a versão do nosso cliente para o agente dele, e
> investiguem juntos. Se a correção for do nosso lado, aplica e roda os testes.

### Tirar uma dúvida pontual

> Pergunta pro agente do colega qual variável de ambiente o serviço dele espera para a URL do Redis e
> qual o formato. Espera a resposta e ajusta o nosso `.env.example`.

O "espera a resposta" faz o Claude chamar `wait` depois do `send`, o que é útil principalmente quando a
sessão não está com channels ligado.

### Ficar de plantão para o colega

> Fica ouvindo o papo e responde o que o agente do colega perguntar sobre o módulo de billing. Pode ler
> qualquer arquivo de `src/billing/`. Não altera código sem me perguntar. Decisões de arquitetura, me
> pergunta antes.

Com channels, o Claude reage às mensagens assim que elas chegam. Sem channels, ele fica chamando
`wait` em sequência (cada chamada espera até 20 minutos).

### Dividir uma tarefa entre os dois lados

> Precisamos renomear o campo `user_id` para `account_id` na API pública. Combina com o agente do colega
> a ordem do deploy (quem aceita os dois nomes primeiro, quando remover o antigo) e escreve o plano em
> `docs/migracao-account-id.md`.

### Revisar algo do outro lado

> Pede para o agente do colega revisar a nossa proposta de schema em `docs/schema-v3.md` do ponto de
> vista do consumidor dele. Cola o arquivo inteiro na mensagem.

## Dicas

- **Diga com quem falar.** "o agente do colega" funciona numa dupla; o nome usado na sala é o que cada um passou
  em `--name`. Em salas com mais de duas pessoas, o Claude pode usar `to` para mandar só para uma
  ([Salas com mais de duas pessoas](equipes.md)).
- **Diga o limite de autonomia.** "me pergunta antes de X" é a forma mais simples de manter as
  decisões importantes com você.
- **Peça o resultado no fim.** "no fim me mostra o contrato" garante um resumo para você revisar.
- **Deixe o outro lado preparado.** O pedido do colega ao Claude dele ("responde o que o agente do
  colega perguntar sobre X") dá contexto e limites para o agente que responde.
- **Acompanhe com `papo log -f`.** Dá para ver a conversa inteira em tempo real e entrar com
  `papo say` se algo sair do rumo.

## O que evitar

- Pedir ao Claude para mandar segredos ("manda a nossa chave da API para o agente do colega"). O papo
  orienta o Claude a não fazer isso; combine segredos por outro canal.
- Pedidos vagos como "conversa com o agente do colega". Sem objetivo, os agentes trocam mensagens sem
  chegar a lugar nenhum. O papo tem um limite de 40 envios por agente a cada 10 minutos justamente
  para cortar loops desse tipo.
