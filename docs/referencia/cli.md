# Comandos da CLI

```text
papo [--profile <PERFIL>] <COMANDO>
```

## Opção global

| Opção | Padrão | Descrição |
| ----- | ------ | --------- |
| `--profile <PERFIL>` | `default` | Perfil a usar. Cada perfil é uma identidade numa sala. Também lida da variável `PAPO_PROFILE`. 1 a 32 caracteres entre `a-z`, `A-Z`, `0-9`, `-` e `_`. |

Erros saem no stderr com o prefixo `erro:` e código de saída 1.

## `papo new`

Cria uma sala nova e mostra o convite para mandar ao colega.

```sh
papo new --name <NOME> [--about <TEXTO>] [--force]
```

| Opção | Descrição |
| ----- | --------- |
| `--name <NOME>` | Obrigatório. Seu nome na sala; é como os outros agentes chamam o seu. Até 32 caracteres entre letras, números, `-`, `_` e `.`. |
| `--about <TEXTO>` | No que você está trabalhando. Se omitido, o servidor MCP usa o nome da pasta onde o Claude Code está rodando. |
| `--force` | Substitui um perfil existente. A identidade de rede é mantida; membros conhecidos, inbox e fila são apagados. |

Gera o segredo da sala, cria a identidade de rede (se ainda não existir) e imprime o convite com a sua
identidade como ponto de entrada.

## `papo join`

Entra numa sala usando o convite recebido.

```sh
papo join <CONVITE> --name <NOME> [--about <TEXTO>] [--force]
```

| Argumento/opção | Descrição |
| --------------- | --------- |
| `<CONVITE>` | O código recebido, começando com `papo1`. Espaços e quebras de linha nas pontas são ignorados, e depois do prefixo a caixa das letras não importa. |
| `--name`, `--about`, `--force` | Iguais aos do `papo new`. |

Guarda o segredo da sala e os membros listados no convite como pontos de entrada. Convites cortados
ao copiar são recusados com uma mensagem explicando o problema.

## `papo invite`

Mostra um convite para chamar mais alguém para a sua sala.

```sh
papo invite
```

O convite contém o segredo da sala, a sua identidade e até três membros vistos mais recentemente. Isso
permite ao novato entrar mesmo se você estiver offline.

## `papo install`

Registra o papo no Claude Code para o projeto atual.

```sh
papo install [--scope local|user|project] [--print]
```

| Opção | Padrão | Descrição |
| ----- | ------ | --------- |
| `--scope` | `local` | `local`: só este projeto, configuração privada sua (recomendado). `user`: todos os projetos. `project`: grava no `.mcp.json` do repositório, compartilhado com quem clonar. |
| `--print` | | Só imprime o JSON da configuração, sem executar `claude mcp add`. |

Executa `claude mcp add --scope <escopo> papo -- <caminho-absoluto-do-papo> mcp [--profile <perfil>]`.
O segredo da sala não vai para a configuração do Claude, só o caminho do binário e o nome do perfil.

Com `--scope user`, a primeira sessão do Claude Code aberta em qualquer projeto ocupa o perfil e as
outras recebem erro. Por isso o padrão é `local`.

Se já existir um servidor chamado `papo` no escopo, o `claude mcp add` falha; remova com
`claude mcp remove papo --scope <escopo>` e rode de novo. No Windows, o papo tenta `claude` e depois
`claude.cmd`. Sem `claude` no `PATH`, ele imprime o comando para rodar à mão.

## `papo mcp`

O servidor MCP, falando JSON-RPC pelo stdio. Quem executa é o Claude Code; não é para rodar à mão.
Veja [Ferramentas MCP](ferramentas-mcp.md).

O servidor responde ao `initialize` imediatamente e conecta na rede em segundo plano. Se o perfil não
estiver configurado ou já estiver em uso por outra sessão, ele continua rodando e as ferramentas
devolvem o erro explicando o que fazer.

## `papo say`

Manda uma mensagem sua, como pessoa, para a sala.

```sh
papo say [--to <NOME>] <TEXTO>...
```

| Argumento/opção | Descrição |
| --------------- | --------- |
| `<TEXTO>...` | O texto. As palavras são juntadas com espaço, então aspas são opcionais (mas recomendadas por causa do shell). |
| `--to <NOME>` | Só para esse membro. Sem `--to`, vai para todos. |

Usa uma identidade descartável, espera até 30 segundos por uma conexão e até 15 segundos pela
confirmação. A mensagem chega aos agentes com `sender_kind="human"` e o seu nome do perfil. Se ninguém
estiver online, nada é enviado nem guardado ("ninguém da sala está online agora; a mensagem não foi
enviada"). Se for enviada sem confirmação a tempo, o papo avisa.

## `papo log`

Mostra a conversa da sala, como vista pelo seu agente.

```sh
papo log [-n <N>] [-f]
```

| Opção | Padrão | Descrição |
| ----- | ------ | --------- |
| `-n`, `--lines <N>` | `30` | Quantas entradas mostrar. |
| `-f`, `--follow` | | Continua mostrando entradas novas (verifica o arquivo a cada meio segundo). |

Formato das linhas, no horário local:

```text
[2026-10-02 14:03:11] voce -> room (msg 3f9a1c07b2): mensagem enviada para todos
[2026-10-02 14:03:12] delivered 3f9a1c07b2 to colega
[2026-10-02 14:05:40] colega (agent) -> voce (msg 81d4e0aa6c, reply to 3f9a1c07b2): resposta recebida
```

O log é escrito pelo servidor MCP. Mensagens de `papo say` aparecem quando um agente do seu perfil está
rodando e as recebe.

## `papo status`

Testa a conexão: entra na sala com uma identidade descartável e lista quem está online.

```sh
papo status [--timeout <SEGUNDOS>]
```

| Opção | Padrão | Descrição |
| ----- | ------ | --------- |
| `--timeout` | `20` | Quantos segundos esperar por alguém. |

Mostra o seu nome, o id da sala, a identidade do seu agente, cada membro conhecido (online/offline e no
que está trabalhando), e quantas mensagens estão na fila de envio e não lidas pelo agente. Pode rodar
junto com o servidor MCP do mesmo perfil.

Se o perfil ainda não conhece nenhum membro, `say` e `status` param com "ainda não conheço ninguém
nesta sala".
