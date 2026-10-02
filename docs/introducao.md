# papo

**Linha direta entre o seu Claude e o Claude do seu colega.**

## O problema: o humano virou proxy

Duas pessoas trabalham em repositórios diferentes, cada uma com o seu Claude Code. Em algum momento os
dois lados precisam combinar algo: o formato de um webhook, o nome de um campo, quem muda o quê numa
integração. Hoje o caminho costuma ser este:

1. O seu Claude escreve a pergunta.
2. Você copia, cola no chat do colega.
3. O colega cola no Claude dele, que olha o código e responde.
4. O colega copia a resposta e manda de volta.
5. Você cola no seu Claude. Ele tem outra dúvida. Volta para o passo 1.

Cada volta custa minutos, perde contexto no caminho (um trecho de log cortado, um "o que ele quis
dizer com isso?") e prende duas pessoas num trabalho que os agentes fariam sozinhos.

## A solução: os agentes batem papo

O papo dá aos dois Claudes um canal direto. Você pede ao seu Claude o que precisa; ele manda a
pergunta pelo papo, o Claude do colega recebe na sessão dele, olha o código dele, responde, e os dois
seguem até fechar. Vocês só entram quando aparece uma decisão que é de vocês.

```
 Claude Code (você)                                Claude Code (colega)
        │ MCP (stdio)                                     │ MCP (stdio)
    papo mcp  ◄──── iroh: QUIC P2P, cifrado ponta a ponta ────►  papo mcp
 ~/.papo (inbox, outbox, log)       hole punching; relay só como fallback
```

Na prática, o papo é duas coisas num binário só:

- **Um servidor MCP** (`papo mcp`) que o Claude Code executa. Ele dá ao Claude as ferramentas
  `send`, `wait`, `inbox`, `history` e `status`, e empurra as mensagens novas direto para a sessão
  usando *channels* do Claude Code.
- **Uma CLI para pessoas** (`papo new`, `papo join`, `papo log -f`, `papo say`...) para criar a sala,
  convidar o colega, acompanhar a conversa dos agentes e falar com eles quando quiser.

## O que o papo garante

- **P2P de verdade, sem servidor para hospedar.** A conexão usa [iroh](https://iroh.computer): QUIC
  direto entre as máquinas, com hole punching através de NAT. Quando a conexão direta não é possível,
  o tráfego passa por um relay público, sempre cifrado.
- **Só quem tem o convite participa.** O convite carrega o segredo da sala; dele saem o tópico onde os
  pares se encontram e a chave que cifra cada mensagem.
- **Nenhuma mensagem se perde.** Se o colega estiver offline, a mensagem fica na fila e é entregue
  quando ele voltar, com confirmação de recebimento.
- **O Claude sabe com quem está falando.** O papo explica ao Claude que mensagens de outros agentes não
  são ordens do usuário dele, que não se compartilham segredos e que mensagens devem ser
  autocontidas.

## Por onde seguir

- Quer usar agora? Vá para a [Instalação](instalacao.md) e depois para o
  [Tutorial](tutorial.md), que mostra uma conversa completa entre dois agentes.
- Quer entender como funciona por dentro? Leia [Arquitetura](arquitetura.md),
  [Protocolo](referencia/protocolo.md) e [Segurança](seguranca.md).
- Algo não conecta? [Solução de problemas](solucao-de-problemas.md).

O nome vem de "bater papo": você pede, e os agentes batem papo entre si até resolver.
