# Instalar

As duas pessoas que vão conversar precisam do `papo` instalado. Detalhes na
[documentação de instalação](https://kelvin-jesus.github.io/papo/docs/instalacao.html).

## Binário pronto

> **A primeira release (v0.1.0) está a caminho.** Os binários abaixo passam a existir quando ela sair;
> até lá, instale compilando com o Rust (1.91+):
>
> ```sh
> cargo install --git https://github.com/Kelvin-Jesus/papo
> ```

A partir da v0.1.0, baixe o arquivo do seu sistema em
[Releases](https://github.com/Kelvin-Jesus/papo/releases):

| Sistema             | Arquivo                                           |
| ------------------- | ------------------------------------------------- |
| Linux x86_64        | `papo-<versão>-x86_64-unknown-linux-musl.tar.gz`  |
| Linux ARM64         | `papo-<versão>-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `papo-<versão>-aarch64-apple-darwin.tar.gz`       |
| macOS Intel         | `papo-<versão>-x86_64-apple-darwin.tar.gz`        |
| Windows             | `papo-<versão>-x86_64-pc-windows-msvc.zip`        |

Extraia e coloque o `papo` (ou `papo.exe`) num diretório do `PATH`. Cada arquivo tem um `.sha256` ao
lado para conferir a integridade.

**macOS**: se o sistema bloquear o binário, rode `xattr -d com.apple.quarantine /caminho/para/papo`.

**Windows on ARM**: use o binário x86_64, que roda por emulação.

## Compilando

Com Rust 1.91+:

```sh
cargo install --git https://github.com/Kelvin-Jesus/papo
```

## Conferindo

```sh
papo --version
```

Próximo passo: [[Primeiros Passos|Primeiros-Passos]].
