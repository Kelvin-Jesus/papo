# Marcos

Marcos pequenos, cada um com um critério de saída que dá para conferir. Um marco só está **feito**
quando o critério passa; "funciona na minha máquina" não conta. O estado de cada linha está em
[Status](status.md).

| # | Marco | Estado |
| - | ----- | ------ |
| M0 | Pesquisa e arquitetura | feito |
| M1 | Núcleo P2P: sala, cifra, gossip, entrega, fila | feito |
| M2 | Servidor MCP e channels | feito no protocolo |
| M3 | Documentação, site e harness | feito (site v2 em andamento) |
| M4 | Testes completos | feito |
| M5 | Release v0.1.0 | feito |
| M6 | Validação com duas sessões reais do Claude Code | em parte |
| M7 | Empacotamento (Homebrew, Scoop, winget) | planejado |

O M5 pode sair antes do M6: o release é útil para o próprio M6 (o colega baixa o binário em vez de
compilar). Nesse caso as notas do release dizem com todas as letras que a validação com sessões reais
ainda não aconteceu.

---

## M0: pesquisa e arquitetura

**Escopo:** descobrir como um servidor MCP empurra mensagens para dentro de uma sessão do Claude Code,
escolher o transporte P2P e fixar as decisões que o resto depende.

**Entregue:** [pesquisa](pesquisa.md) sobre *channels*, timeouts de MCP, iroh e iroh-gossip;
ADRs [0001](../adr/0001-o-transporte-e-iroh-com-gossip.md),
[0002](../adr/0002-mensagens-chegam-por-push-com-pull-de-reserva.md),
[0004](../adr/0004-o-servidor-mcp-e-escrito-a-mao.md) e
[0005](../adr/0005-entrega-pelo-menos-uma-vez-com-ack-e-fila.md).

**Critério de saída:** cada decisão com alternativas descartadas e o motivo. Cumprido.

## M1: núcleo P2P

**Escopo:** `room`, `proto`, `store`, `net`, `node`. Dois nós na mesma sala trocam mensagens com
confirmação, e nada se perde se um deles está fora do ar.

**Critério de saída:**

| Critério | Verificação | Resultado |
| -------- | ----------- | --------- |
| Mensagem entregue e confirmada | `tests/node.rs`, rede local | passa |
| Fila sobrevive a reinício do remetente | `tests/node.rs` | passa |
| Endereçamento com `to` | `tests/node.rs` | passa |
| Dois processos se acham pela internet pública | `cargo test --test mcp -- --ignored` | passa em cerca de 6 s |

O último critério falhou na primeira tentativa: a sala nunca se formava. A causa era o iroh-gossip
não discar de novo um par cuja primeira discagem falhou. O contorno virou a
[ADR 0003](../adr/0003-o-papo-disca-os-pares-antes-do-gossip.md) e uma linha em
[Problemas conhecidos](problemas-conhecidos.md).

## M2: servidor MCP e channels

**Escopo:** `papo mcp` por stdio com as ferramentas `send`, `wait`, `inbox`, `history` e `status`,
push por `notifications/claude/channel` e instruções de colaboração para o agente.

**Critério de saída:**

| Critério | Verificação | Resultado |
| -------- | ----------- | --------- |
| Handshake e capability `claude/channel` | `tests/mcp.rs` | passa |
| Negociação nunca passa de `2025-11-25` | `tests/mcp.rs` e teste unitário | passa |
| Notificação de channel com `meta` válido | e2e público | passa |
| `wait` cancelável e com timeout | `tests/mcp.rs` | passa |
| Push visto dentro de uma sessão real do Claude Code | M6 | pendente |

## M3: documentação, site e harness

**Entregue:** livro em mdBook ([este](../introducao.md)), ADRs, wiki do GitHub em `wiki/`, base OKF em
`knowledge/`, `AGENTS.md`, skills em `.agents/skills/`, site v1 no GitHub Pages, diagramas Mermaid e
esta camada de engenharia.

**Critério de saída:** `mdbook build docs` sem avisos, links internos resolvendo, site e livro no ar.
Cumprido. O site v2 (marca, design system e demo interativa) segue como trabalho separado.

## M4: testes completos

**Escopo:** todos os tipos principais de teste: unitários, propriedade (proptest), robustez com
entradas arbitrárias e alvos de `cargo fuzz`, integração com nós reais, contrato do MCP, CLI com
snapshots, e2e, doctests, benchmarks, cobertura medida com limiar no CI, mutação e cargo-deny.

**Critério de saída:** a suíte inteira passa nos três sistemas, a cobertura das linhas da biblioteca
fica acima do limiar definido no CI, e cada bug encontrado tem commit próprio e linha em
[Problemas conhecidos](problemas-conhecidos.md).

**Estado:** feito em 2026-10-02 e integrado na `main`: 146 testes offline, 94,3% das linhas cobertas, quatro bugs corrigidos com commit e teste próprios. Detalhes em [Status](status.md#testes-m4-2026-10-02).

## M5: release v0.1.0

**Critério de saída:**

1. M4 integrado em `main` com o CI verde.
2. Notas em [`docs/releases/v0.1.0.md`](../releases/v0.1.0.md) revisadas e fora do estado de rascunho.
3. Tag `v0.1.0` publicada com o OK do mantenedor; o workflow `release` gera os 5 arquivos e os SHA-256.
4. Cada arquivo baixado da página de Releases confere com `sha256sum -c`, e `papo --version` roda em
   Linux, macOS e Windows.

**Estado:** feito em 2026-10-02. Tag `v0.1.0` com as notas como corpo da release; os 5 arquivos
conferem com os SHA-256, os binários de Linux, macOS e Windows são do tipo certo, o de Linux x86_64
rodou uma troca real pela internet, e a imagem do GHCR baixa sem login nas duas arquiteturas. Rodar
os binários de macOS e Windows numa máquina desses sistemas fica para quem tiver uma (o CI já roda
toda a suíte de testes neles).

## M6: validação com duas sessões reais

**Escopo:** a pergunta que importa: os dois Claudes se entendem sem humano no meio?

**Critério de saída:**

| Critério | Como verificar |
| -------- | -------------- |
| Duas pessoas, duas máquinas, redes diferentes | anotar os tipos de rede (casa, escritório, 4G) |
| Push visto nas duas sessões | a mensagem aparece sem o usuário digitar |
| Modo pull visto numa sessão sem a flag | `wait` traz a resposta |
| Uma tarefa real resolvida pelos agentes | o resumo final bate com o que os dois humanos esperavam |
| Nenhum vazamento de segredo, nenhum loop de "ok/obrigado" | ler o `papo log` inteiro |

O resultado vai para [Status](status.md) com data, e as falhas viram itens do [Roadmap](roadmap.md).

**Estado em 2026-10-02:** em parte. Duas sessões reais do Claude Code (sonnet) conversaram pelo papo
no modo pull, na mesma máquina, e seguiram as regras de colaboração; ver
[Validação com o Claude Code](validacao-claude-code.md), repetível com
`scripts/validate-claude-code.sh`. Faltam o push por channels (confirmação interativa) e duas
máquinas em redes diferentes.

## M7: empacotamento

**Escopo:** instalar sem baixar arquivo à mão: tap do Homebrew, bucket do Scoop, manifesto do winget
e, talvez, um script de instalação. Depende do M5.
