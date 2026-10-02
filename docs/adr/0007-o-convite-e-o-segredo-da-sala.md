# O convite é o segredo da sala

2026-10-02

O papo não tem contas nem servidor de autenticação. Quem tem o segredo de 32
bytes da sala é membro, e o convite (`papo1` + base32 do segredo e de até
quatro endpoints) é a forma de passá-lo adiante. Cada frame é selado com
XChaCha20-Poly1305 usando uma chave derivada do segredo, além do TLS do QUIC,
então relays e nós intermediários só veem bytes cifrados, e quem não tem o
convite não lê nem injeta mensagens.

O segredo fica só em `~/.papo/profiles/<perfil>/profile.json` (0600). A
configuração do Claude Code recebe apenas `papo mcp --profile <perfil>`, para
que o segredo nunca acabe num `.mcp.json` versionado.

## Consequências

Simples de usar: basta mandar um código por um canal privado. Em troca, não há
revogação individual: para tirar alguém, cria-se uma sala nova e manda-se o
convite só para quem fica. Todos os membros confiam uns nos outros; os frames
não são assinados por membro, então um membro poderia se passar por outro
nome dentro da sala.
