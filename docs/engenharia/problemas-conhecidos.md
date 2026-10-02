# Problemas conhecidos e lições

Problemas que custaram tempo de verdade, a causa e a regra que saiu de cada um. Os abertos vêm
primeiro. Acrescente uma linha quando algo surpreender você. Armadilhas de ferramentas e do fluxo de
trabalho (hooks, worktrees, mdBook) ficam em [Armadilhas](../contribuidores/armadilhas.md); a
evidência técnica de cada uma das regras do produto está em `knowledge/gotchas/`.

## Abertos

| Problema | Contorno ou plano |
| -------- | ----------------- |
| *Channels* do Claude Code é *research preview*: a flag e o protocolo podem mudar | O papo também funciona no modo pull (`wait`, `inbox`); acompanhar a documentação de channels a cada versão do Claude Code |
| O Claude Code descarta as notificações de channel em silêncio quando channels está desligado, e o servidor não tem como saber | Mensagens empurradas continuam não lidas até `wait`/`inbox` ou uma resposta com `reply_to` ([ADR 0002](../adr/0002-mensagens-chegam-por-push-com-pull-de-reserva.md)) |
| Em organizações Team/Enterprise, *Channels* vem bloqueado até um Owner habilitar | Modo pull; ou pedir ao admin para ligar *Channels* (ou `channelsEnabled: true` nas configurações gerenciadas) |
| A flag `--dangerously-load-development-channels` mostra um diálogo de confirmação a cada sessão | Esperar o empacotamento como plugin ([Roadmap](roadmap.md)) |
| Em salas com mais de duas pessoas, uma mensagem sem `to` sai da fila no primeiro ack | Usar `to` para garantir a entrega a alguém específico |
| Frames não são assinados por membro: um membro pode se passar por outro nome dentro da sala | Todos na sala têm o convite e confiam uns nos outros ([ADR 0007](../adr/0007-o-convite-e-o-segredo-da-sala.md)); assinatura por membro está no roadmap |
| Não há revogação: quem tem o convite entra | Criar uma sala nova (`papo new --force`) e mandar o convite só para quem fica |
| Relays públicos da n0 têm limite de uso | `PAPO_RELAY` aponta para um `iroh-relay` próprio |
| Binários sem assinatura: Gatekeeper no macOS, SmartScreen no Windows | `xattr -d com.apple.quarantine ./papo`; no Windows, "Executar assim mesmo" |
| Só um servidor MCP por perfil: uma segunda sessão do Claude Code no mesmo perfil recebe erro | Um perfil por projeto (`papo install --profile <nome>`) |
| Travessia de NAT entre redes diferentes ainda não foi testada (o e2e roda na mesma máquina) | Parte do M6 |
| A wiki do GitHub não sincroniza até a primeira página ser salva pela interface | Salvar qualquer página em `/wiki/_new` e rodar o workflow `wiki` de novo |
| `papo say` e `papo status` terminam imprimindo `papo: gossip subscription closed` no stderr mesmo quando tudo deu certo (visto em 2026-10-02) | Inofensivo: é o laço de eventos do nó efêmero avisando o fim da assinatura no encerramento normal. Correção: não avisar quando o fim vem de um `shutdown` pedido |

## Resolvidos (mantenha as regras)

| O que aconteceu | Causa | Regra |
| --------------- | ----- | ----- |
| A sala nunca se formava pela internet: o segundo membro ficava sozinho para sempre | No iroh-gossip 0.101, um par passado como bootstrap cuja primeira discagem falha fica `Pending` no ator do gossip; `join_peers` depois disso só enfileira mensagens e nunca disca de novo. O caso comum: discar um colega que abriu a sessão há um segundo e ainda não publicou o endereço ("No addressing information available") | Nunca passar pares ao gossip como bootstrap; o papo disca com o ALPN do gossip e entrega a conexão pronta via `Gossip::handle_connection` ([ADR 0003](../adr/0003-o-papo-disca-os-pares-antes-do-gossip.md)) |
| O primeiro contato levava 12 s | Tentativas a cada 10 s fixos; a primeira sempre perdia a corrida contra a publicação do endereço | Backoff que começa em 1 s e dobra até 10 s; voltou para cerca de 6 s no e2e |

## Pegos na revisão, antes de acontecer

| Risco | Causa | Regra |
| ----- | ----- | ----- |
| Duplicata se o processo morresse entre gravar o inbox e o log | A deduplicação era reconstruída só a partir do log | Os ids do inbox também entram no conjunto de vistos ao subir |
| Registro órfão de cancelamento numa chamada MCP muito rápida | A tarefa podia terminar antes de o handle ser guardado no mapa | Criar a tarefa segurando o lock do mapa |
| Nó "zumbi" em silêncio se a assinatura do gossip morresse | Nada sinalizava o fim do laço de eventos | `Node::is_healthy`; `status` e `send` avisam para reiniciar a sessão |
| Duas sessões no mesmo perfil disputando a mesma identidade na rede | Mesmo endpoint id em dois processos | Trava exclusiva do perfil (`File::try_lock`) no `papo mcp` |

## Regras que vieram da pesquisa

| Fato | Fonte | Regra |
| ---- | ----- | ----- |
| O tamanho máximo padrão de mensagem do iroh-gossip é 4096 bytes, pouco para trechos de código | `DEFAULT_MAX_MESSAGE_SIZE` no código do iroh-gossip | `Gossip::builder().max_message_size(64 KiB)`; corpo limitado a 48 KiB |
| Servidor que negocia a revisão `2026-07-28` do MCP não é registrado como channel pelo Claude Code | documentação de channels | Negociar no máximo `2025-11-25` |
| Chaves do `meta` que não são identificadores são descartadas em silêncio | documentação de channels | Só `[A-Za-z0-9_]`: `from`, `msg_id`, `sender_kind`, `reply_to`, `to` |

Detalhes e fontes de cada fato: [Pesquisa](pesquisa.md).
