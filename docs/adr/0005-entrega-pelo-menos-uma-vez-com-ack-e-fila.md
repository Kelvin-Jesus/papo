# A entrega é pelo menos uma vez, com ack e fila

2026-10-02

O colega pode estar offline, o processo pode morrer no meio, e o gossip não
garante entrega. Uma pergunta perdida entre dois agentes é pior que uma
repetida, então a entrega é pelo menos uma vez:

- `send` grava a mensagem na outbox **antes** do broadcast;
- quem recebe grava inbox e log e **só então** responde com `Ack`;
- o remetente tira a mensagem da outbox no primeiro ack (do destinatário,
  quando há `to`);
- a outbox é reenviada quando um vizinho conecta e a cada 30 s enquanto há
  conexão;
- o destinatário deduplica pelo id (que sobrevive a reinícios, lido do log e
  da inbox) e reconhece de novo as duplicatas, então um ack perdido se cura
  sozinho.

## Consequências

Nenhuma mensagem se perde por o par estar offline ou por queda do processo.
O custo é tráfego repetido enquanto algo está na fila.

Numa sala com mais de duas pessoas, uma mensagem sem `to` sai da fila no
primeiro ack, então quem estava offline pode não recebê-la. Para garantir a
entrega a alguém específico, use `to`.
