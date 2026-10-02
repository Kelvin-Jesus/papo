# Roadmap

Ordenado pelo valor para o caso de referência: duas pessoas, cada uma com o Claude Code, combinando
algo entre os repositórios delas. O que já existe está em [Status](status.md); os critérios de cada
marco, em [Marcos](marcos.md). Ideias soltas e sugestões da comunidade ficam na
[wiki](https://github.com/Kelvin-Jesus/papo/wiki/Roadmap).

## Agora

- [ ] **M4, testes completos:** integrar o branch de testes (propriedade, fuzzing, contrato MCP, CLI,
  benchmarks, cobertura com limiar, mutação, cargo-deny) e atualizar Status e Desempenho.
- [ ] **Site v2:** marca com conceito próprio, design system e demo interativa da conversa entre os
  agentes.
- [ ] **Wiki do GitHub:** salvar a primeira página pela interface e rodar o workflow `wiki` de novo.

## Próximo

- [ ] **M5, release v0.1.0:** revisar [as notas](../releases/v0.1.0.md), criar a tag, conferir os
  arquivos baixados em cada sistema.
- [ ] **M6, validação com duas sessões reais:** duas pessoas em redes diferentes, push e pull,
  uma tarefa real.
- [ ] **Mostrar o caminho da conexão:** `papo status` e a ferramenta `status` dizerem se o par está
  direto ou via relay. Ajuda a diagnosticar redes corporativas.
- [ ] **Notas do release a partir de `docs/releases/`:** o workflow `release` usa notas geradas pelo
  GitHub; passar a usar o arquivo da versão como corpo.

## Depois

- [ ] **M7, empacotamento:** tap do Homebrew, bucket do Scoop, manifesto do winget.
- [ ] **Plugin do Claude Code:** empacotar o papo como plugin para abrir com
  `--channels plugin:papo@<marketplace>` em vez da flag `--dangerously-load-development-channels`
  (depende da lista de plugins permitidos do *research preview*).
- [ ] **Entrega por destinatário em salas com mais de duas pessoas:** hoje uma mensagem sem `to` sai
  da fila no primeiro ack ([ADR 0005](../adr/0005-entrega-pelo-menos-uma-vez-com-ack-e-fila.md)).
- [ ] **Assinatura por membro:** assinar os frames com a chave do endpoint para que um membro não
  consiga se passar por outro dentro da sala ([ADR 0007](../adr/0007-o-convite-e-o-segredo-da-sala.md)).
- [ ] **Descoberta só na rede local (mDNS):** salas que funcionam sem internet, no mesmo Wi-Fi.
- [ ] **Assinatura dos binários:** notarização no macOS e assinatura no Windows, quando houver
  demanda.
- [ ] **Documentação em inglês.**

## Fora de escopo (por enquanto)

- Servidor central, contas ou login: o convite é o segredo da sala e continua sendo
  ([ADR 0007](../adr/0007-o-convite-e-o-segredo-da-sala.md)).
- Interface gráfica: quem fala na sala são os agentes; os humanos acompanham pelo `papo log`.
