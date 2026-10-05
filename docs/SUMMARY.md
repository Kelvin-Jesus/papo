# Sumário

[Introdução](introducao.md)

# Começando

- [Instalação](instalacao.md)
- [Tutorial: dois agentes combinando um contrato](tutorial.md)

# Guias

- [Pedidos ao Claude que funcionam bem](guias/pedidos-ao-claude.md)
- [Usando sem channels](guias/sem-channels.md)
- [Várias salas com perfis](guias/varias-salas.md)
- [Salas com mais de duas pessoas](guias/equipes.md)
- [Relay próprio](guias/relay-proprio.md)
- [Docker](guias/docker.md)

# Referência

- [Comandos da CLI](referencia/cli.md)
- [Ferramentas MCP](referencia/ferramentas-mcp.md)
- [Configuração e arquivos](referencia/configuracao.md)
- [Protocolo](referencia/protocolo.md)

# Por dentro

- [Arquitetura](arquitetura.md)
- [Segurança](seguranca.md)
- [Glossário](glossario.md)

# Ajuda

- [Solução de problemas](solucao-de-problemas.md)
- [Perguntas frequentes](faq.md)
- [Contribuindo](contribuindo.md)

# Engenharia

- [Status](engenharia/status.md)
- [Marcos](engenharia/marcos.md)
- [Roadmap](engenharia/roadmap.md)
- [Problemas conhecidos](engenharia/problemas-conhecidos.md)
- [Pesquisa](engenharia/pesquisa.md)
- [Desempenho](engenharia/desempenho.md)
- [Diagramas](engenharia/diagramas.md)
- [Desenvolvimento](engenharia/desenvolvimento.md)
- [Validação com o Claude Code](engenharia/validacao-claude-code.md)
- [Notas de release]()
  - [v0.2.0](releases/v0.2.0.md)
  - [v0.1.0](releases/v0.1.0.md)

# Para contribuidores e agentes

- [O que todo contribuidor precisa saber](contribuidores/README.md)
  - [Trabalhando com o mantenedor](contribuidores/trabalhando-com-o-mantenedor.md)
  - [Agentes em paralelo](contribuidores/agentes-em-paralelo.md)
  - [Hábitos do projeto](contribuidores/habitos.md)
  - [Armadilhas](contribuidores/armadilhas.md)
  - [Ambiente de desenvolvimento](contribuidores/ambiente.md)
  - [Releases](contribuidores/releases.md)
  - [Site e documentação](contribuidores/site-e-docs.md)

# Decisões de arquitetura (ADRs)

- [Índice das decisões](adr/README.md)
  - [0001: O transporte é iroh com gossip](adr/0001-o-transporte-e-iroh-com-gossip.md)
  - [0002: Mensagens chegam por push, com pull de reserva](adr/0002-mensagens-chegam-por-push-com-pull-de-reserva.md)
  - [0003: O papo disca os pares antes do gossip](adr/0003-o-papo-disca-os-pares-antes-do-gossip.md)
  - [0004: O servidor MCP é escrito à mão](adr/0004-o-servidor-mcp-e-escrito-a-mao.md)
  - [0005: Entrega pelo menos uma vez, com ack e fila](adr/0005-entrega-pelo-menos-uma-vez-com-ack-e-fila.md)
  - [0006: Um binário estático por plataforma](adr/0006-um-binario-estatico-por-plataforma.md)
  - [0007: O convite é o segredo da sala](adr/0007-o-convite-e-o-segredo-da-sala.md)
  - [0008: Docs em mdBook e site no GitHub Pages](adr/0008-docs-em-mdbook-e-site-no-github-pages.md)
