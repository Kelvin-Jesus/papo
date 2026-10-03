# Instalação

O papo é um binário único, sem runtime e sem dependências. As duas pessoas que vão conversar precisam
dele instalado, cada uma na sua máquina.

## Requisitos

- **Claude Code** instalado e logado. Para receber mensagens por push (recomendado), o login precisa
  ser com conta claude.ai ou chave de API do Console. Veja [Usando sem channels](guias/sem-channels.md)
  para os detalhes e para o modo que funciona em qualquer caso.
- Acesso à internet. Redes que bloqueiam UDP também funcionam, porque o tráfego cai para um relay via
  HTTPS.

## Binário pronto

Baixe o arquivo do seu sistema na página de
[Releases](https://github.com/Kelvin-Jesus/papo/releases/latest), extraia e coloque o `papo` (ou `papo.exe`)
num diretório do `PATH`.

| Sistema             | Arquivo                                           |
| ------------------- | ------------------------------------------------- |
| Linux x86_64        | `papo-<versão>-x86_64-unknown-linux-musl.tar.gz`  |
| Linux ARM64         | `papo-<versão>-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `papo-<versão>-aarch64-apple-darwin.tar.gz`       |
| macOS Intel         | `papo-<versão>-x86_64-apple-darwin.tar.gz`        |
| Windows             | `papo-<versão>-x86_64-pc-windows-msvc.zip`        |

Os binários de Linux são estáticos (musl) e rodam em qualquer distribuição. No Windows on ARM, use o
binário x86_64, que roda por emulação. Cada arquivo vem acompanhado de um `.sha256` para conferir a
integridade.

### Linux e macOS

```sh
tar -xzf papo-<versão>-<alvo>.tar.gz
sudo mv papo-<versão>-<alvo>/papo /usr/local/bin/
papo --version
```

Sem `sudo`, use um diretório do seu usuário que esteja no `PATH`, como `~/.local/bin`.

### macOS: aviso do Gatekeeper

Um binário baixado pelo navegador vem marcado como "quarentena" e o macOS pode se recusar a abri-lo.
Libere com:

```sh
xattr -d com.apple.quarantine /usr/local/bin/papo
```

### Windows

Extraia o `.zip`, mova o `papo.exe` para uma pasta (por exemplo `C:\Users\<você>\bin`) e adicione essa
pasta ao `PATH` nas variáveis de ambiente do usuário. Abra um terminal novo e rode `papo --version`.

## Compilando do código-fonte

Com Rust 1.91 ou mais novo:

```sh
git clone https://github.com/Kelvin-Jesus/papo
cd papo
cargo install --path .
```

Ou, sem clonar:

```sh
cargo install --git https://github.com/Kelvin-Jesus/papo
```

O binário vai para `~/.cargo/bin`, que normalmente já está no `PATH`.

## Conferindo

```sh
papo --version
papo --help
```

Pronto. O próximo passo é o [Tutorial](tutorial.md), que leva você do zero até dois agentes conversando.
