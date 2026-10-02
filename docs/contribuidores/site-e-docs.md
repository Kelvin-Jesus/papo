# Site e documentação

Onde mora cada pedaço da documentação publicada e como ele chega ao ar.

| Peça | Fonte | Publicação |
| ---- | ----- | ---------- |
| Página do projeto | `site/` (HTML, CSS e JS estáticos) e `assets/logo/` | workflow `pages`, raiz de https://kelvin-jesus.github.io/papo/ |
| Livro | `docs/` (mdBook, `src = "."`, tema em `docs/theme/`) | workflow `pages`, em `/docs/` |
| Wiki | `wiki/` | workflow `wiki`, espelha tudo na wiki do GitHub |
| README | `README.md` | o próprio GitHub |
| Decisões | `docs/adr/` | no livro e no GitHub |
| Base para agentes | `knowledge/` (OKF) | só no repositório |

## Regras

- **Um assunto, uma fonte.** O README apresenta, o livro aprofunda, a wiki junta dicas de quem usa.
  Quando o mesmo fato aparece em dois lugares, um deles é um link
  ([ADR 0008](../adr/0008-docs-em-mdbook-e-site-no-github-pages.md)).
- **Push em `main` publica.** O workflow `pages` dispara quando `docs/`, `site/` ou `assets/` mudam;
  confira o build local antes.
- **Mermaid no livro.** O `docs/theme/mermaid-init.js` troca os blocos ```` ```mermaid ```` por
  diagramas, carregando o Mermaid do CDN só nas páginas que têm diagrama e seguindo o tema claro ou
  escuro do livro. O GitHub renderiza Mermaid sozinho no README e nos `.md`.
- **Glossário por inclusão.** `docs/glossario.md` inclui o `CONTEXT.md` (sem o título); não copie o
  glossário para outro lugar.
- **Wiki é a pasta `wiki/`.** Não edite a wiki pela interface do GitHub: o próximo sync sobrescreve.
- **Site em redesenho.** A página do projeto está sendo redesenhada (marca, design system, demo
  interativa); o mantenedor acompanha a evolução num canvas privado do Claude Design. Mudanças visuais
  que vão para o ar precisam do "sim" dele.

## Verificação

```sh
mdbook build docs -d /tmp/papo-book && python3 scripts/check-book-links.py /tmp/papo-book
python3 scripts/check-mermaid.py README.md docs/arquitetura.md docs/engenharia/diagramas.md
```
