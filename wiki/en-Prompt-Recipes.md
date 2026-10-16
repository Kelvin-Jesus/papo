# Prompt recipes

[[Português|Receitas-de-Prompts]] · English

Good requests say **who to talk to**, **what to solve** and **where to stop and check with you**.
More recipes and explanations are in the guide
[Pedidos ao Claude que funcionam bem](https://kelvin-jesus.github.io/papo/docs/guias/pedidos-ao-claude.html)
(in Portuguese).

This page is open to contributions: do you have a request that worked well? Add it here.

## Agree on a contract

> Work out with my colleague's agent the contract of the `order.shipped` event that our service
> publishes and theirs consumes. Use what already exists in `src/events/`. For breaking changes, ask
> me first. At the end, show me the final contract.

## Investigate an error together

> The integration test against my colleague's service is failing with 422 on `POST /v2/invoices`. Send
> the full error, the payload and our client version to their agent, and investigate together.

## Ask a question and wait

> Ask my colleague's agent which environment variable their service expects for the Redis URL. Wait
> for the answer and update our `.env.example`.

## Stay on call

> Keep listening on papo and answer whatever my colleague's agent asks about the billing module. Do not
> change code without asking me.

## Plan a change on both sides

> We need to rename `user_id` to `account_id` in the public API. Agree with my colleague's agent on
> the deploy order and write the plan in `docs/migracao-account-id.md`.

## Avoid

- Asking it to send secrets (keys, tokens, `.env`). Share secrets through another channel.
- Requests with no goal, such as "talk to my colleague's agent".
