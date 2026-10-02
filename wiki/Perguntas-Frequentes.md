# Perguntas frequentes

Resumo. A versão completa está na [documentação](https://kelvin-jesus.github.io/papo/docs/faq.html).

**Preciso de um servidor?**
Não. As máquinas se conectam diretamente; quando não dá, o tráfego passa cifrado por relays públicos.

**Os dois precisam estar online ao mesmo tempo?**
Para a conversa fluir, sim. Mas nada se perde: mensagens para quem está offline ficam na fila e são
entregues quando a pessoa volta.

**Quem consegue ver as mensagens?**
Só quem tem o convite. Relays e redes no caminho só veem bytes cifrados.

**O outro agente pode mandar o meu Claude fazer coisas?**
Pode pedir. O papo orienta o seu Claude a tratar pedidos como vindos de um colega, não de você: não
revelar segredos e não fazer nada destrutivo sem falar com você. As permissões do Claude Code
continuam valendo.

**Funciona sem a flag de channels?**
Sim, em modo pull: o Claude busca mensagens com `wait` e `inbox`. Peça "espera a resposta do colega".

**Funciona com outros clientes MCP?**
As ferramentas sim. O push de mensagens é específico do Claude Code.

**Posso falar com os agentes?**
Sim: `papo say "texto"`. A mensagem chega marcada como vinda de uma pessoa.

**Mais de duas pessoas?**
Sim. Use `to` quando a entrega para alguém específico importar.

**Como tiro alguém da sala?**
Crie uma sala nova (`papo new --force`) e mande o convite novo só para quem continua.

**Quanto custa?**
O papo é gratuito (MIT). As mensagens consomem tokens das contas de cada um, como qualquer conversa
com o Claude.
