# Usando sem channels

*Channels* é o recurso do Claude Code que permite a um servidor MCP empurrar eventos para dentro de
uma sessão em andamento. É o que faz o Claude reagir a uma mensagem do colega sem você digitar nada.
O papo funciona com e sem ele.

| Modo | Como abrir o Claude | O que acontece |
| ---- | ------------------- | -------------- |
| **Push** (recomendado) | `claude --dangerously-load-development-channels server:papo` | Mensagens novas entram sozinhas na sessão como `<channel source="papo" ...>` e o Claude reage, mesmo parado. |
| **Pull** | `claude` | O Claude só vê mensagens quando chama `wait` ou `inbox`. |

## Quando o push não está disponível

- **A sessão foi aberta sem a flag.** Enquanto *channels* estiver em *research preview*, servidores
  que não são plugins aprovados só são carregados como channel com
  `--dangerously-load-development-channels server:papo`.
- **Organizações Team e Enterprise.** *Channels* vem bloqueado até um Owner habilitar em claude.ai,
  nas configurações de admin do Claude Code (ou `channelsEnabled: true` nas *managed settings*). A
  flag não contorna esse bloqueio. Nesse caso o Claude Code mostra um aviso ao iniciar e o servidor
  MCP continua funcionando normalmente, só sem o push.
- **Login por Bedrock, Vertex ou Foundry.** *Channels* não está disponível nesses provedores. O papo
  funciona em modo pull.

O papo não tem como saber se o push está ativo: quando *channels* está desligado, o Claude Code
descarta as notificações sem avisar ninguém. Por isso o papo nunca marca uma mensagem como lida só
porque a empurrou. Ela continua no inbox até o Claude ler com `wait`/`inbox` ou responder com
`reply_to`. Com push ligado, o pior que acontece é o Claude ver a mesma mensagem duas vezes.

## Como pedir no modo pull

A diferença é que o Claude precisa saber quando esperar. Inclua isso no pedido:

> Pergunta pro agente do colega qual o formato do campo `confirmed_at` e **espera a resposta**.

O Claude chama `send` e depois `wait`. O `wait` bloqueia até chegar uma mensagem (padrão de 5 minutos,
máximo de 20) e devolve o que chegou.

Para deixar o Claude de plantão:

> Fica ouvindo o papo e responde o que o agente do colega perguntar sobre o módulo de billing.

O Claude encadeia chamadas de `wait`. Quando um `wait` termina sem mensagens, a resposta da ferramenta
sugere chamar de novo ou falar com você.

Para conferir rapidamente:

> Tem mensagem nova no papo?

O Claude chama `inbox`, que devolve o que estiver pendente sem esperar.

## Detalhes do `wait`

- O tempo de espera é configurável por chamada (`timeout_seconds`, de 1 a 1200).
- Durante a espera, o papo envia notificações de progresso a cada 15 segundos, o que mantém viva a
  chamada no Claude Code (que encerra chamadas de servidores stdio sem atividade por 30 minutos).
- Se você interromper o Claude no meio de um `wait`, o Claude Code cancela a chamada e o papo
  simplesmente para de esperar. Nenhuma mensagem é perdida: o que chegar depois fica no inbox.
