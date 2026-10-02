# O transporte é iroh com gossip

2026-10-02

Os agentes de duas pessoas precisam se falar direto, de redes domésticas ou de
escritório, sem ninguém hospedar servidor, e o produto tem de ser um binário
nativo para Linux, macOS e Windows. Por isso o papo é escrito em Rust sobre o
[iroh](https://iroh.computer) 1.x (QUIC, discagem pela chave pública, hole
punching, relays da n0 como reserva cifrada e descoberta de endereço por
DNS/pkarr), com o iroh-gossip para distribuir as mensagens dentro da sala.

## Alternativas consideradas

- **go-libp2p**: compila cruzado com facilidade, mas atravessar NAT de forma
  confiável exige relays de circuito e ajuste fino de DCUtR, e os relays
  públicos têm recursos limitados.
- **Servidor WebSocket próprio**: o mais simples e sempre alcançável, mas
  alguém tem de manter o servidor no ar e deixa de ser P2P.
- **Hyperswarm (Node)**: ótimo hole punching por DHT, mas exige runtime
  JavaScript, o que inviabiliza o binário nativo.

## Consequências

Funciona atrás de NAT sem infraestrutura própria; quando a conexão direta
falha, o tráfego passa cifrado pelos relays gratuitos (e limitados) da n0, e
`PAPO_RELAY` permite usar um `iroh-relay` próprio.

O gossip é melhor esforço, então as garantias de entrega moram no `node.rs`
(ver [0005](0005-entrega-pelo-menos-uma-vez-com-ack-e-fila.md)). O gossip
também tem um defeito de rediscagem que contornamos
([0003](0003-o-papo-disca-os-pares-antes-do-gossip.md)).
