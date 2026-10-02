# Desempenho e medições

O papo troca mensagens de texto entre agentes, então o que importa não é vazão: é **quanto tempo até
dois membros se acharem**, **quanto custa um processo `papo mcp` parado** (ele fica aberto o dia todo
dentro do Claude Code) e **quanto cada mensagem pesa**. Esta página diz como medir cada coisa e traz
os números medidos. Datar cada número; refazer a medida quando a parte do código correspondente mudar.

Máquina das medidas locais: Linux x86_64, 12 núcleos, Rust 1.98, binário de release (`lto = "thin"`,
`codegen-units = 1`, `strip`). Rede: internet doméstica; o relay escolhido foi `use1-1.relay.n0.iroh.link`.

## Números (2026-10-02)

| Medida | Valor | Observação |
| ------ | ----- | ---------- |
| e2e público completo (`cargo test --test mcp -- --ignored`) | 6,04 s | convite, conexão, `send`, push, resposta com `reply_to`, `wait`, `history`; build de debug |
| `papo say` com o par já online | 3,8 a 3,9 s (3 execuções) | sobe um endpoint novo, publica o endereço, disca, entrega, espera o ack e encerra |
| `papo say` logo depois de o par subir | 4,7 s | inclui a corrida contra a publicação do endereço do par |
| `papo status` | 4,5 s | inclui 3 s de espera fixa para as mensagens de presença chegarem |
| `papo mcp` responder ao `initialize` e sair no fim do stdin | 45 ms | o nó sobe em paralelo; o `initialize` não espera a rede |
| Memória de um `papo mcp` parado (RSS) | 21,8 MB | 10 s depois de subir, 14 threads; 21,9 MB depois de receber uma mensagem |
| CPU de um `papo mcp` parado | 0,5 % | média dos primeiros 16 s, incluindo a subida |
| Binário de release, Linux x86_64 (glibc) | 17,4 MB (17 359 088 bytes) | sem símbolos |
| Artefatos do CI (arquivo compactado + SHA-256) | 5,6 a 7,1 MB | macOS ARM64 o menor, Linux x86_64 musl o maior (run 37067969904) |
| Build de release do zero | 316 s | inclui todas as dependências, com outros builds rodando na máquina ao mesmo tempo; um build anterior sem concorrência levou 255 s |
| Testes de integração do nó (5 testes, relay local) | 1,9 s | `cargo test --test node` |
| Convite | 108 caracteres | `papo1` + base32 do segredo e de um endpoint |

### Tamanho de um frame

| Frame | Bytes na rede | Composição |
| ----- | ------------- | ---------- |
| `msg` com corpo vazio | 203 | 163 de JSON + 24 de nonce + 16 de tag |
| `msg` com `to` e `reply_to` | 238 | |
| `ack` | 80 | 40 de JSON + 40 de selagem |
| `hello` | cerca de 200 | depende do nome e do `about` |

O corpo vai em UTF-8 sem escapes além dos do JSON (aspas, barra invertida, quebras de linha). Limites:
corpo de 48 KiB e frame de 64 KiB ([Protocolo](../referencia/protocolo.md)). Por cima disso existe o
enquadramento do iroh-gossip e o do QUIC, que não foram medidos.

## Como medir

### Tempo até conectar

```sh
cargo build --release
B=target/release/papo
INV=$(PAPO_HOME=/tmp/m/a $B new --name ana | grep -o 'papo1[a-z0-9]*')
PAPO_HOME=/tmp/m/b $B join "$INV" --name bob

# terminal 1: o servidor da Ana, com stdin aberto
PAPO_HOME=/tmp/m/a $B mcp

# terminal 2: cole o initialize no terminal 1, depois meça o say
time PAPO_HOME=/tmp/m/b $B say "teste"
```

O `say` sobe uma identidade descartável a cada execução, então o tempo dele inclui escolher relay,
publicar o endereço e discar: é o pior caso do primeiro contato, repetido.

### Memória e CPU parado

Com o servidor do terminal 1 rodando há alguns segundos:

```sh
grep -E 'VmRSS|VmHWM|Threads' /proc/$(pgrep -x papo)/status
ps -o pcpu,etime -p $(pgrep -x papo)
```

Use `pgrep -x papo`: com `timeout` ou `tail` no meio, `pgrep -f` pega o processo errado.

### e2e público

```sh
cargo test --test mcp -- --ignored
```

O teste espera o Bob ver a Ana online consultando `status` a cada 2 s, então o resultado anda em
degraus de 2 s: não serve para medir diferenças menores que isso. Para o tempo de conexão em si, use o
`say` acima.

### Benchmarks

Benchmarks com criterion (selagem, codificação de frames, convite) estão sendo adicionados num branch
separado. Quando ele for integrado, rode `cargo bench` e traga os números para esta página com a data.

## O que olhar quando algo ficar lento

- **Conexão demorando mais de 15 s:** o nó pode estar caindo no relay ou não achando o endereço. Veja
  `PAPO_LOG=iroh=info,iroh::address_lookup=debug` ([Desenvolvimento](desenvolvimento.md#diagnóstico-de-rede)).
- **`send` respondendo "queued" com o par online:** o ack não chegou em 8 s (`ACK_TIMEOUT`). A mensagem
  continua na outbox e é reenviada a cada 30 s.
- **Memória subindo com o tempo:** o conjunto de ids vistos cresce com o log (um id de 10 caracteres
  por mensagem recebida) e é reconstruído a partir do `log.jsonl` ao subir. Em conversas muito longas,
  vale medir.
