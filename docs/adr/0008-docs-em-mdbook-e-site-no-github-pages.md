# Docs em mdBook e site no GitHub Pages

2026-10-02

A documentação longa (tutorial, guias, referência, arquitetura, ADRs) vive em
Markdown dentro de `docs/` e vira um livro com o mdBook, a ferramenta padrão
do ecossistema Rust. A página do projeto é HTML e CSS estáticos em `site/`. Um
único workflow (`pages.yml`) monta as duas coisas em `_site/` (site na raiz,
livro em `/docs/`) e publica no GitHub Pages.

O glossário do livro inclui o `CONTEXT.md` em vez de copiá-lo, e a wiki fica
para conteúdo leve e editável pela comunidade (receitas de pedidos, perguntas
frequentes), apontando para o livro quando o assunto é detalhado.

## Consequências

Uma fonte por assunto: o README apresenta, o livro aprofunda, a wiki junta
dicas. Nenhuma dependência de Node ou de gerador de site além do mdBook, que o
workflow baixa em versão fixa.
