# Um binário estático por plataforma

2026-10-02

Quem usa o papo é quem já usa o Claude Code, em qualquer sistema, e o colega
não deveria precisar instalar runtime nem compilar nada. O release gera um
único executável por plataforma: Linux x86_64 e ARM64 estáticos com musl
(compilados com `cargo zigbuild`, porque o ring precisa de um toolchain C
cruzado), macOS Intel e Apple Silicon, e Windows x64. Cada arquivo sai com o
SHA-256.

## Consequências

Os binários de Linux rodam em qualquer distribuição, independente da versão da
glibc. Windows em ARM usa o binário x64 por emulação.

Os binários não são assinados: o macOS pode bloquear o download pelo
Gatekeeper (`xattr -d com.apple.quarantine`) e o Windows pode mostrar o aviso
do SmartScreen. Assinatura fica para quando houver demanda.
