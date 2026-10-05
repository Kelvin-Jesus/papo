#!/bin/sh
# Instala o binário do papo a partir das releases do GitHub (Linux e macOS).
#
#   curl -fsSL https://raw.githubusercontent.com/Kelvin-Jesus/papo/main/scripts/install.sh | sh
#   curl -fsSL .../install.sh | sh -s -- --dir /usr/local/bin --version v0.1.0
#
# Detecta sistema e arquitetura, baixa o arquivo da release, confere o .sha256 e copia o
# `papo` para o diretório escolhido (padrão: ~/.local/bin, sem sudo). POSIX sh de propósito:
# roda pelo `| sh` em qualquer Linux ou macOS. O nome dos arquivos acompanha o passo
# "Package" de .github/workflows/release.yml; mudou lá, muda aqui.
set -eu

REPO="Kelvin-Jesus/papo"
dir="${HOME}/.local/bin"
version=""

die() {
  echo "erro: $*" >&2
  exit 1
}

while [ $# -gt 0 ]; do
  case "$1" in
    --dir)
      [ $# -ge 2 ] || die "--dir precisa de um caminho"
      dir="$2"
      shift 2
      ;;
    --version)
      [ $# -ge 2 ] || die "--version precisa de uma tag (ex.: v0.1.0)"
      version="$2"
      shift 2
      ;;
    -h | --help)
      echo "uso: install.sh [--dir <diretório>] [--version <vX.Y.Z>]"
      exit 0
      ;;
    *) die "opção desconhecida: $1 (use --help)" ;;
  esac
done

if command -v curl >/dev/null 2>&1; then
  fetch() { curl -fsSL -o "$2" "$1"; }
  latest_url() { curl -fsSLI -o /dev/null -w '%{url_effective}' "$1"; }
elif command -v wget >/dev/null 2>&1; then
  fetch() { wget -q -O "$2" "$1"; }
  # wget prints the redirect chain on stderr; the last Location is the release tag.
  latest_url() { wget --spider -S "$1" 2>&1 | sed -n 's/^ *[Ll]ocation: *//p' | tail -n 1 | tr -d '\r'; }
else
  die "preciso de curl ou wget"
fi

os="$(uname -s)"
arch="$(uname -m)"
case "$arch" in
  x86_64 | amd64) arch="x86_64" ;;
  aarch64 | arm64) arch="aarch64" ;;
  *) die "arquitetura sem binário pronto: $arch (compile com: cargo install --git https://github.com/$REPO)" ;;
esac
case "$os" in
  Linux) target="${arch}-unknown-linux-musl" ;;
  Darwin)
    # A shell under Rosetta reports x86_64 on Apple Silicon; the native binary is better.
    if [ "$arch" = "x86_64" ] && [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = "1" ]; then
      arch="aarch64"
    fi
    target="${arch}-apple-darwin"
    ;;
  *) die "sistema sem suporte neste script: $os (no Windows, veja a seção de instalação do README)" ;;
esac

if [ -z "$version" ]; then
  # /releases/latest redirects to /releases/tag/<tag>; no API call, so no rate limit.
  version="$(latest_url "https://github.com/${REPO}/releases/latest")"
  version="${version##*/}"
  case "$version" in
    v*) ;;
    *) die "não consegui descobrir a última versão (use --version vX.Y.Z)" ;;
  esac
fi

name="papo-${version}-${target}"
base="https://github.com/${REPO}/releases/download/${version}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "baixando ${name}.tar.gz…" >&2
fetch "${base}/${name}.tar.gz" "${tmp}/${name}.tar.gz" || die "download falhou: ${base}/${name}.tar.gz"
fetch "${base}/${name}.tar.gz.sha256" "${tmp}/${name}.tar.gz.sha256" || die "download do .sha256 falhou"

expected="$(cut -d ' ' -f 1 <"${tmp}/${name}.tar.gz.sha256")"
if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "${tmp}/${name}.tar.gz" | cut -d ' ' -f 1)"
elif command -v shasum >/dev/null 2>&1; then
  actual="$(shasum -a 256 "${tmp}/${name}.tar.gz" | cut -d ' ' -f 1)"
else
  die "preciso de sha256sum ou shasum para conferir o download"
fi
if [ -z "$expected" ] || [ "$expected" != "$actual" ]; then
  die "checksum não confere (esperado ${expected}, obtido ${actual})"
fi

tar -xzf "${tmp}/${name}.tar.gz" -C "$tmp"
mkdir -p "$dir"
# Copy to a temporary name and rename, so a running `papo mcp` keeps its old inode.
cp "${tmp}/${name}/papo" "${dir}/.papo.new"
chmod 0755 "${dir}/.papo.new"
mv -f "${dir}/.papo.new" "${dir}/papo"

echo "papo ${version} instalado em ${dir}/papo" >&2
"${dir}/papo" --version

case ":${PATH}:" in
  *":${dir}:"*) ;;
  *)
    echo "" >&2
    echo "atenção: ${dir} não está no PATH. Adicione ao seu shell, por exemplo:" >&2
    echo "  echo 'export PATH=\"${dir}:\$PATH\"' >> ~/.profile" >&2
    ;;
esac
