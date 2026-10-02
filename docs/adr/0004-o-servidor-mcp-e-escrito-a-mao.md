# O servidor MCP é escrito à mão

2026-10-02

A superfície é pequena (cinco ferramentas), e as duas coisas que importam
aqui ficam fora do que os SDKs de MCP modelam bem: a capability experimental
`claude/channel` com a notificação própria, e chamadas de ferramenta em long
poll que precisam ser canceláveis (`notifications/cancelled`) e manter vivo o
timer de inatividade do cliente (`notifications/progress`). Por isso o
`src/mcp.rs` implementa JSON-RPC 2.0 por stdio diretamente, uma mensagem por
linha.

## Consequências

Controle total sobre negociação de versão, push e cancelamento, e menos
dependências e tempo de compilação. Em troca, mudanças no protocolo MCP são
nossas para acompanhar; `tests/mcp.rs` fala com o binário como o Claude Code
fala e é a rede de segurança.

stdout pertence ao JSON-RPC: qualquer diagnóstico vai para stderr.
