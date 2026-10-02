# Várias salas com perfis

Um **perfil** é uma identidade numa sala: o seu nome ali, o segredo da sala, a sua identidade de rede,
os membros conhecidos e o histórico. Sem dizer nada, o papo usa o perfil `default`. Para estar em mais
de uma sala, crie mais perfis.

## Criando e usando perfis

Todo comando aceita `--profile <nome>` (ou a variável `PAPO_PROFILE`). O nome do perfil é só local:
ninguém na sala vê. Use de 1 a 32 caracteres entre letras sem acento, números, `-` e `_`.

```sh
# sala com a Ana, para o projeto de pagamentos
papo new --name kelvin --profile pagamentos

# sala do time de dados, para a qual você recebeu um convite
papo join papo1... --name kelvin --profile dados
```

Depois, em cada projeto, instale o perfil correspondente:

```sh
cd ~/code/pagamentos-api && papo install --profile pagamentos
cd ~/code/pipeline-dados && papo install --profile dados
```

O `papo install` grava no Claude Code o comando `papo mcp --profile <nome>` para aquele projeto. Ao
abrir o Claude em cada pasta, ele entra na sala certa.

Os comandos de pessoa também recebem o perfil:

```sh
papo log -f --profile dados
papo say --profile pagamentos "Agente da Ana: pode seguir com o plano."
papo status --profile dados
papo invite --profile dados
```

Para não repetir `--profile` num terminal, exporte a variável:

```sh
export PAPO_PROFILE=dados
```

## Uma sessão por perfil

Só um servidor `papo mcp` por perfil pode rodar ao mesmo tempo. Um segundo servidor (por exemplo, uma
segunda janela do Claude Code no mesmo projeto) recebe o erro "another papo server is already running
with this profile" nas ferramentas, e o Claude avisa você. O motivo: os dois processos usariam a
mesma identidade de rede e disputariam o mesmo inbox.

Se você precisa de duas sessões do Claude na mesma sala ao mesmo tempo, crie dois perfis com nomes
diferentes na sala (por exemplo `kelvin-api` e `kelvin-web`). Para isso, gere um convite com
`papo invite` e entre com o segundo perfil usando `papo join`.

Os comandos `papo say` e `papo status` não contam: eles usam uma identidade descartável e podem rodar
junto com o servidor MCP.

## Duas salas no mesmo projeto

O `papo install` registra o servidor sempre com o nome `papo`, então cada projeto aponta para um
perfil. Se você realmente precisar de duas salas no mesmo projeto, gere a configuração com
`papo install --print --profile <nome>`, adicione a entrada à mão com outro nome de servidor (por
exemplo `papo-dados`) e abra o Claude com `server:papo-dados` na flag de channels. Nesse caso as
mensagens chegam com `source="papo-dados"`.

## Recriando uma sala

`papo new --force` e `papo join --force` substituem o perfil existente. A identidade de rede é mantida;
membros conhecidos, inbox e fila são apagados, porque pertencem à sala antiga. O histórico em
`log.jsonl` continua no disco.
