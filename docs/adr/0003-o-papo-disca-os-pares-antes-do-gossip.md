# O papo disca os pares antes de envolver o gossip

2026-10-02

No iroh-gossip 0.101, um par passado como bootstrap que não pode ser discado na
primeira tentativa fica num estado `Pending` dentro do ator do gossip, e
chamadas posteriores de `join_peers` só enfileiram mensagens sem discar de
novo. O caso comum é discar um colega que abriu a sessão há um segundo e ainda
não publicou o endereço. Resultado observado: a sala nunca se forma. Uma
conexão *recebida* desse par limpa o estado, e o protocolo do gossip é
simétrico (streams unidirecionais nos dois sentidos).

Por isso o papo assina o tópico sem nenhum bootstrap. O laço de manutenção
disca cada membro conhecido com o ALPN do gossip (`Endpoint::connect`, timeout
de 15 s), entrega a conexão a `Gossip::handle_connection` como se fosse
recebida e só então chama `join_peers` para aquele membro. Enquanto está
sozinho, tenta de novo com backoff exponencial de 1 s até 10 s.

## Consequências

O primeiro contato no caso típico fecha em cerca de 2 s (eram 12 s com
tentativas fixas a cada 10 s). Nunca passe pares para o gossip como bootstrap.

Se uma versão futura do iroh-gossip corrigir a rediscagem, isto continua
funcionando, mas vale revisitar para simplificar.
