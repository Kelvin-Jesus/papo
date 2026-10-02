# Status

O que funciona, como foi verificado e o que falta. Atualize esta página a cada mudança que mexe numa
linha da tabela. Definição dos marcos: [Marcos](marcos.md). Planos: [Roadmap](roadmap.md).

Legenda:

- **verificado**: exercitado de verdade, fora de um teste com rede simulada (internet pública, CI
  nos três sistemas, comando rodado à mão).
- **testado**: coberto por teste automatizado com rede local (relay em processo), sem internet.
- **documentado**: descrito, sem verificação automática.
- **não verificado**: ainda não foi exercitado.

## Marcos

| Marco | Estado |
| ----- | ------ |
| M0 Pesquisa e arquitetura | feito: [pesquisa](pesquisa.md), [arquitetura](../arquitetura.md), ADRs 0001 a 0005 |
| M1 Núcleo P2P | feito: sala, cifra, gossip, entrega com ack e fila offline, reconexão própria |
| M2 MCP e channels | feito no protocolo; falta a sessão real do Claude Code (é o M6) |
| M3 Docs e site | feito: livro, ADRs, wiki, OKF, site v1; site v2 em andamento |
| M4 Testes completos | feito: 146 testes offline + 1 pela internet, 94,3% das linhas cobertas, fuzzing, mutação e cargo-deny no CI |
| M5 Release v0.1.0 | planejado: builds das 5 plataformas já passam no CI, falta a tag |
| M6 Validação com duas sessões reais | planejado |
| M7 Empacotamento | planejado |

## Núcleo P2P (2026-10-02)

| Área | Estado | Evidência |
| ---- | ------ | --------- |
| Segredo da sala, convite `papo1`, derivação de tópico/chave/id | testado | 6 testes unitários em `src/room.rs` (ida e volta, convite truncado, ruído de apps de chat) |
| Selagem XChaCha20-Poly1305 com AAD `papo/v1` | testado | `room.rs`: abre só com a mesma sala, rejeita frame adulterado |
| Frames `hello`, `msg`, `ack` e limites (corpo 48 KiB, frame 64 KiB) | testado | 4 testes em `src/proto.rs` |
| Entrega com ack, gravada antes de confirmar | testado | `tests/node.rs::message_is_delivered_acked_and_logged` |
| Fila offline que sobrevive a reinício do remetente | testado | `tests/node.rs::queued_message_survives_sender_restart_and_reaches_late_peer` |
| Mensagem com `to` só para o destinatário | testado | `tests/node.rs::addressed_message_only_reaches_the_addressee` |
| Responder com `reply_to` marca como lido | testado | `tests/node.rs::replying_marks_that_peers_earlier_messages_as_read` |
| Nó efêmero da CLI (`say`, `status`) | testado | `tests/node.rs::human_cli_can_talk_but_is_not_remembered_as_member` |
| Conexão pela internet pública (DNS/pkarr da n0 e relays) | verificado | `cargo test --test mcp -- --ignored` passou em todas as execuções locais de 2026-10-02 e no CI (job `e2e-public-network`); números em [Desempenho](desempenho.md) |
| Reconexão com discagem própria ([ADR 0003](../adr/0003-o-papo-disca-os-pares-antes-do-gossip.md)) | verificado | antes do contorno a sala nunca se formava em 90 s; com tentativas fixas a cada 10 s, 12 s; com backoff a partir de 1 s, 6 s |
| Travessia de NAT entre redes diferentes | não verificado | os dois processos do e2e rodam na mesma máquina (ou no mesmo runner do CI); falta um teste entre duas redes de verdade |
| Caminho usado (direto ou relay) | não verificado | o papo não expõe isso ainda; ver [Roadmap](roadmap.md) |

## Servidor MCP e Claude Code (2026-10-02)

| Área | Estado | Evidência |
| ---- | ------ | --------- |
| `initialize`, capability `claude/channel`, negociação que nunca passa de `2025-11-25` | testado | `tests/mcp.rs::speaks_mcp_and_advertises_the_channel_capability` |
| `tools/list` com `send`, `wait`, `inbox`, `history`, `status` | testado | mesmo teste |
| Perfil não configurado: o servidor sobe e explica o que fazer | testado | `tests/mcp.rs::unconfigured_profile_still_starts_and_explains_setup` |
| Um servidor por perfil (trava do arquivo) | testado | `tests/mcp.rs::second_server_on_the_same_profile_is_refused` |
| `wait` com timeout e cancelamento por `notifications/cancelled` | testado | `tests/mcp.rs::wait_times_out_and_can_be_cancelled` |
| Notificação `notifications/claude/channel` com `from`, `msg_id`, `sender_kind` | verificado | o e2e público recebe a notificação pelo stdout do binário real |
| Push numa sessão real do Claude Code (`--dangerously-load-development-channels server:papo`) | não verificado | o formato segue a documentação de channels; ver [Pesquisa](pesquisa.md) |
| Modo pull (`wait`, `inbox`) numa sessão real do Claude Code | não verificado | só exercitado pelo cliente de teste, que fala MCP como o Claude Code |
| `papo install` executando `claude mcp add` | não verificado | `install --print` foi conferido à mão; o comando real não foi rodado para não mexer na configuração do mantenedor |
| Qualidade das instruções de colaboração (o modelo segue as regras?) | não verificado | depende do M6 |

## CLI (2026-10-02)

| Área | Estado | Evidência |
| ---- | ------ | --------- |
| `new`, `join`, `invite`, `install --print` | verificado | rodados à mão com `PAPO_HOME` temporário |
| `say` entregue a um servidor MCP rodando | verificado | à mão: "entregue a colega", e o servidor emitiu a notificação com `sender_kind: human` |
| `status` lista quem está online e no que trabalha | verificado | à mão contra um servidor rodando |
| `log` e `log -f` com horário local | verificado | à mão |

## Distribuição e documentação (2026-10-02)

| Item | Estado | Evidência |
| ---- | ------ | --------- |
| CI em Linux, macOS e Windows (fmt, clippy `-D warnings`, testes) | verificado | run 37067916552 do workflow `ci` |
| Builds de release das 5 plataformas | verificado (build) | run 37067969904 do workflow `release` por `workflow_dispatch`; os binários de macOS e Windows não foram executados |
| Release publicado (tag) | planejado | nenhuma tag ainda; é o M5 |
| Site e livro no GitHub Pages | verificado | https://kelvin-jesus.github.io/papo/ e `/docs/` respondem 200 |
| Diagramas Mermaid no livro | verificado | 7 diagramas renderizados num Chromium headless sem erro |
| Wiki do GitHub sincronizada a partir de `wiki/` | bloqueado | o GitHub só cria o repositório da wiki depois que a primeira página é salva pela interface; até lá o workflow `wiki` falha |
| Base de conhecimento OKF em `knowledge/` | verificado | validador da skill OKF sem erros nem avisos |

## Em andamento

- **Site v2:** redesenho da página do projeto com conceito de marca, design system e demo interativa.

## Testes (M4, 2026-10-02)

| Tipo | Quantos | Onde |
| ---- | ------- | ---- |
| Unitários (biblioteca e binário) | 33 e 9 | nos módulos de `src/` |
| Doctests | 3 | API pública de `room` e `proto` |
| Propriedade (proptest) | 13 | convite, cifra, frames, nomes, endereçamento |
| Robustez (bytes arbitrários) | 6 | decodificação de convite, frame, cifra e linhas do MCP |
| Armazenamento | 15 | escrita atômica, permissões 0600, trava, arquivos corrompidos |
| Nó | 5 + 16 cenários | entrega, fila, reinício, várias salas, multi-hop, queda sem shutdown |
| MCP e contrato do MCP | 5 + 20 | JSON-RPC, cancelamento, progresso, freio, snapshots com insta, JSON Schema das ferramentas |
| CLI | 19 | comandos, erros e saídas |
| e2e local, dois processos (feature `test-network`) | 2 | dois `papo mcp` e a CLI num relay local |
| e2e pela internet | 1 (ignorado por padrão) | `cargo test --test mcp -- --ignored` |

Cobertura (cargo-llvm-cov): 94,3% das linhas e 93,1% das regiões; por módulo, proto 100, mcp 98,2,
room 97,9, net 97,8, node 92,7, store 90,6 e main 87,9. O CI falha abaixo de 90% das linhas. Três
alvos de `cargo fuzz` rodaram limpos localmente (31 milhões, 2,3 milhões e 2,6 milhões de execuções)
e rodam no CI com tempo curto; o cargo-mutants roda por agendamento. Benchmarks com criterion em
`benches/` (`cargo bench`).

## Não feito

Ver [Roadmap](roadmap.md): validação com duas sessões reais, travessia de NAT entre redes,
empacotamento (Homebrew, Scoop, winget), plugin do Claude Code, assinatura dos binários.
