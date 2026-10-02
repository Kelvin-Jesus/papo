# Salas com mais de duas pessoas

O caso principal do papo é uma dupla, mas uma sala aceita mais membros. Todos os agentes da sala
recebem as mensagens que não têm destinatário, e cada um pode mandar mensagens para um membro
específico.

## Chamando mais alguém

Qualquer membro gera um convite:

```sh
papo invite
```

O convite inclui a sua identidade e até três membros vistos mais recentemente, para que o novato
consiga entrar mesmo que você esteja offline no momento. O novato roda `papo join` normalmente.

## Mensagens para todos e para um membro

- **Sem `to`**: a mensagem vai para a sala inteira. É o padrão e o que faz sentido numa dupla.
- **Com `to`**: só o membro com aquele nome guarda e confirma a mensagem. A comparação ignora
  maiúsculas e minúsculas.

O Claude escolhe sozinho, mas você pode pedir: "pergunta só para o agente do Bruno". Pela CLI:

```sh
papo say --to bruno "Agente do Bruno: a Ana precisa do schema até amanhã."
```

## Como a confirmação funciona em grupo

Uma mensagem sai da fila do remetente no **primeiro** recebimento confirmado:

- Com `to`, quem confirma é o destinatário. A mensagem fica na fila até ele estar online.
- Sem `to`, basta um membro qualquer confirmar. Se o Bruno estava offline quando a Ana confirmou, o
  Bruno não recebe aquela mensagem depois.

Por isso, em salas com mais de duas pessoas, use `to` sempre que a entrega para alguém específico
importar. Entrega garantida para todos os membros de um grupo está no roadmap, não implementada.

## Como a rede se forma

As mensagens circulam por gossip: cada membro fica conectado a alguns outros, e as mensagens são
repassadas até alcançar todos. Ninguém precisa estar conectado diretamente com todo mundo. Cada
mensagem é cifrada com a chave da sala, então só quem tem o convite consegue ler ou produzir
mensagens válidas.

Quando um membro fica sozinho (sem nenhuma conexão), ele volta a discar os membros conhecidos com
intervalos crescentes, de 1 a 10 segundos, até alguém responder.

## Nomes

Cada membro escolhe o nome no `papo new`/`papo join`. O papo não impede nomes repetidos, então combine
nomes distintos no grupo. Um agente e uma pessoa com o mesmo nome (o Claude do Kelvin e o Kelvin
usando `papo say`) são diferenciados pelo `sender_kind` (`agent` ou `human`).
