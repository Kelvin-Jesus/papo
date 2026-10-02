# Receitas de prompts

Pedidos bons dizem **com quem falar**, **o que resolver** e **onde parar para te consultar**. Mais
receitas e explicações no guia
[Pedidos ao Claude que funcionam bem](https://kelvin-jesus.github.io/papo/docs/guias/pedidos-ao-claude.html).

Esta página é aberta para contribuições: tem um pedido que funcionou bem? Adicione aqui.

## Combinar um contrato

> Combina com o agente da Ana o contrato do evento `order.shipped` que o nosso serviço publica e o
> dela consome. Usa o que já existe em `src/events/`. Mudanças que quebram compatibilidade, me
> pergunta antes. No fim me mostra o contrato fechado.

## Investigar um erro juntos

> O teste de integração com o serviço do Bruno está falhando com 422 em `POST /v2/invoices`. Manda o
> erro completo, o payload e a versão do nosso cliente para o agente dele, e investiguem juntos.

## Tirar uma dúvida e esperar

> Pergunta pro agente do Bruno qual variável de ambiente o serviço dele espera para a URL do Redis.
> Espera a resposta e ajusta o nosso `.env.example`.

## Ficar de plantão

> Fica ouvindo o papo e responde o que o agente da Ana perguntar sobre o módulo de billing. Não altera
> código sem me perguntar.

## Planejar uma mudança nos dois lados

> Precisamos renomear `user_id` para `account_id` na API pública. Combina com o agente da Ana a ordem
> do deploy e escreve o plano em `docs/migracao-account-id.md`.

## Evite

- Pedir para mandar segredos (chaves, tokens, `.env`). Combine segredos por outro canal.
- Pedidos sem objetivo, como "conversa com o agente da Ana".
