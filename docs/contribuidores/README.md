# O que todo contribuidor e agente precisa saber

A memória compartilhada de quem trabalha no papo, pessoas e agentes: as lições que custaram tempo e
não aparecem no código. Leia depois do
[`AGENTS.md`](https://github.com/Kelvin-Jesus/papo/blob/main/AGENTS.md) e antes da primeira mudança.

| Página | Leia quando |
| ------ | ----------- |
| [Trabalhando com o mantenedor](trabalhando-com-o-mantenedor.md) | sempre: idioma, autonomia, o que precisa de "sim" antes |
| [Agentes em paralelo](agentes-em-paralelo.md) | sempre: outra sessão costuma estar mexendo no mesmo repositório |
| [Hábitos do projeto](habitos.md) | antes de dar uma mudança por terminada |
| [Armadilhas](armadilhas.md) | algo "deveria funcionar" e não funciona |
| [Ambiente de desenvolvimento](ambiente.md) | montando a máquina, rodando ferramentas, logs |
| [Releases](releases.md) | publicando uma versão |
| [Site e documentação](site-e-docs.md) | mexendo no site, no livro, na wiki ou nos diagramas |

Como isto se relaciona com o resto:

- O `AGENTS.md` tem os comandos, o mapa dos arquivos, as invariantes e a tabela do que manter em
  sincronia. Estas páginas não repetem; explicam o porquê e acrescentam o que não cabe lá.
- A [camada de engenharia](../engenharia/status.md) é a documentação do produto: status, marcos,
  problemas conhecidos, pesquisa, desempenho. Um bug e a regra que saiu dele vão para
  [Problemas conhecidos](../engenharia/problemas-conhecidos.md); um hábito de trabalho ou uma armadilha
  de ferramenta vem para cá.
- `knowledge/` é a base OKF para agentes (conceitos, APIs, protocolo, playbooks), em inglês.
- `CONTEXT.md` é a linguagem do domínio; `docs/adr/` guarda as decisões.

Por que aqui e não na wiki do GitHub: a wiki é para quem usa o papo (receitas de pedidos, perguntas
frequentes) e é espelhada inteira a partir de `wiki/`. Estas páginas são para quem muda o código,
viajam junto com ele no mesmo commit e têm os links conferidos pelo build do livro.

Mantenha vivo:

- Aprendeu algo que não é óbvio e que o próximo vai tropeçar? Acrescente na página certa, no mesmo
  commit do código. Um fato num lugar só, onde vão procurar; nos outros, um link.
- Corrija ou apague o que se mostrar errado. Date o que pode envelhecer ("medido em 2026-10-02").
- Este repositório é público: nada de e-mails, senhas, chaves, convites de salas reais ou caminhos
  pessoais.
