# Relay próprio

Na maior parte das redes, os dois papos se conectam diretamente: o iroh faz hole punching através dos
NATs e o tráfego vai de máquina para máquina. Quando isso não é possível (UDP bloqueado, NAT simétrico
nos dois lados, proxy corporativo), o tráfego passa por um **relay**.

Por padrão, o papo usa os relays públicos mantidos pela n0, a empresa por trás do iroh. Eles são
gratuitos e têm limite de uso. O relay só repassa bytes: as conexões são cifradas de ponta a ponta
pelo QUIC, e cada mensagem ainda é cifrada com a chave da sala.

## Quando vale ter um relay próprio

- A rede bloqueia os relays públicos.
- O uso é intenso e esbarra nos limites dos relays públicos.
- A política da empresa exige que o tráfego passe por infraestrutura própria.

## Como configurar

1. Suba um `iroh-relay`, o servidor de relay do projeto [iroh](https://github.com/n0-computer/iroh).
   Siga a documentação do iroh para a versão que você for usar: ele precisa de um domínio com TLS.
2. Aponte o papo para ele com a variável `PAPO_RELAY`, nas máquinas de **todos** os membros da sala:

   ```sh
   export PAPO_RELAY=https://relay.exemplo.com
   ```

3. Como quem executa o `papo mcp` é o Claude Code, a variável precisa estar no ambiente do servidor
   MCP. A forma mais simples é registrar o servidor com ela:

   ```sh
   claude mcp add --scope local papo -e PAPO_RELAY=https://relay.exemplo.com -- papo mcp
   ```

   Se o papo já estava registrado no projeto, remova antes com `claude mcp remove papo --scope local`.
   Outra opção é exportar a variável no shell antes de abrir o Claude Code. Para `papo say` e
   `papo status`, basta a variável estar exportada no terminal.

Com `PAPO_RELAY` definido, o papo usa só esse relay. A descoberta de endereços continua sendo feita
pelo DNS público da n0, que publica a forma de alcançar cada membro.

## Conferindo

Rode com diagnóstico ligado:

```sh
PAPO_LOG=info papo status
```

Nos logs aparece o relay escolhido como "home relay". Se a URL estiver errada, o papo para na hora com
"PAPO_RELAY is not a valid URL".
