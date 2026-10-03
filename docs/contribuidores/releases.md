# Releases

A mecânica (versão no `Cargo.toml`, tag, workflow, conferência dos arquivos) está em
[Desenvolvimento](../engenharia/desenvolvimento.md#releases); esta página é sobre fazer direito.

## Publicando uma versão

1. **Notas para quem usa.** Escreva `docs/releases/vX.Y.Z.md`: o que muda para as pessoas que usam o
   papo, em pt-BR, com um bloco `<details>` em inglês, mais uma seção "Bom saber" com os limites que
   continuam valendo. Nunca uma lista de commits, nunca jargão interno. Diga o que foi verificado com
   sessões reais do Claude Code e o que não foi. Modelo: [v0.1.0](../releases/v0.1.0.md).
2. **OK do mantenedor.** Publicar precisa de "sim"
   ([Trabalhando com o mantenedor](trabalhando-com-o-mantenedor.md)).
3. **Versão no código e no site.** Suba `version` no `Cargo.toml` (e o `Cargo.lock`), a constante
   `VERSAO` em `site/app.js` e os links de download do README, do botão do hero e da aba de sistemas
   em `site/index.html`: os nomes dos arquivos levam a versão (`papo-vX.Y.Z-<alvo>`).
4. **Tag.** Commit e push de `main` com o `quality gate` verde, depois
   `git tag -a vX.Y.Z -m "..." && git push origin vX.Y.Z`. O workflow `release` recusa a tag se
   `docs/releases/vX.Y.Z.md` não existir, ainda disser "rascunho" ou tiver campos por preencher;
   depois compila os 5 alvos e publica com os `.sha256` e as notas como corpo da release. O workflow
   `docker` publica a imagem multi-arquitetura em `ghcr.io/kelvin-jesus/papo`.
5. **Confira o que foi publicado.** Baixe os arquivos da página de Releases, rode `sha256sum -c` e
   `papo --version`; puxe a imagem (`docker run --rm ghcr.io/kelvin-jesus/papo:X.Y.Z --version`). Anote
   em [Status](../engenharia/status.md).

## Regras

- **Tag publicada é final.** Não apague nem mova tags. Se o release falhou num commit com tag, corrija
  e publique a próxima versão de correção.
- **Teste o build antes da tag.** `gh workflow run release.yml --ref main` compila tudo sem publicar.
- **`--locked`.** O CI usa `Cargo.lock` como está; versão nova no `Cargo.toml` exige `cargo build`
  local antes do commit para atualizar o lock.
- **Runners do CI são mais lentos.** Um teste que depende de tempo e passa local pode falhar lá. Deixe-o
  determinístico (conte eventos em vez de correr contra o relógio) em vez de aumentar o timeout.
- **Binários sem assinatura.** Enquanto não houver assinatura, as notas dizem como liberar no macOS
  (`xattr`) e no Windows (SmartScreen).
