# papo

**Linha direta entre o seu Claude e o Claude do seu colega.**

Quando dois agentes precisam combinar algo (o formato de uma API, um contrato de evento, quem muda o
quê), as pessoas costumam virar proxy: copiam a pergunta de um Claude, colam no chat, o colega cola no
Claude dele e copia a resposta de volta. O papo acaba com esse vai e vem. Os dois Claudes conversam
direto pelo papo, se entendem e só chamam vocês quando precisam de uma decisão.

- Binário único para Linux, macOS e Windows, sem servidor para hospedar.
- Conexão P2P cifrada de ponta a ponta via [iroh](https://iroh.computer), com relay só como fallback.
- Mensagens chegam sozinhas na sessão do Claude Code (*channels*), e nada se perde se o colega estiver
  offline.

## Links

- **Site do projeto**: <https://kelvin-jesus.github.io/papo/>
- **Documentação completa**: <https://kelvin-jesus.github.io/papo/docs/>
- **Downloads**: <https://github.com/Kelvin-Jesus/papo/releases/latest> (v0.1.0) · [[Instalar]]
- **Código e issues**: <https://github.com/Kelvin-Jesus/papo>

## Nesta wiki

- [[Instalar]]: binários por sistema e compilação.
- [[Primeiros Passos|Primeiros-Passos]]: do zero a dois agentes conversando.
- [[Receitas de Prompts|Receitas-de-Prompts]]: pedidos ao Claude que funcionam bem.
- [[Perguntas Frequentes|Perguntas-Frequentes]]
- [[Problemas Comuns|Problemas-Comuns]]
- [[Roadmap]]: ideias para o futuro.

A wiki é um resumo e um espaço para receitas da comunidade. A referência completa (comandos,
ferramentas MCP, protocolo, segurança) está na [documentação](https://kelvin-jesus.github.io/papo/docs/).
