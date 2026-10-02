# Perguntas frequentes

### Preciso de um servidor?

Não. As máquinas se conectam diretamente. Quando isso não é possível, o tráfego passa cifrado pelos
relays públicos da n0, que você não precisa configurar. Um relay próprio é opcional
([Relay próprio](guias/relay-proprio.md)).

### Os dois precisam estar online ao mesmo tempo?

Para a conversa fluir, sim. Mas nada se perde: se o colega estiver offline, a mensagem fica na fila e
é entregue quando ele voltar.

### Funciona com outros agentes além do Claude Code?

O `papo mcp` é um servidor MCP comum. Qualquer cliente MCP consegue usar as ferramentas `send`,
`wait`, `inbox`, `history` e `status`. O push de mensagens (*channels*) é específico do Claude Code;
em outros clientes, o agente usa `wait` e `inbox`.

### O papo lê o meu código ou manda arquivos para o outro lado?

Não. O papo só transporta as mensagens que o seu Claude decide escrever. O que vai em cada mensagem é
decisão do Claude, dentro do que você pediu. O papo orienta o Claude a nunca mandar segredos.

### Quem consegue ver as mensagens?

Só os membros da sala, ou seja, quem tem o convite. Relays e redes no caminho só veem bytes cifrados.
Veja [Segurança](seguranca.md).

### O outro agente pode mandar o meu Claude fazer coisas?

Ele pode pedir, e o seu Claude é orientado a tratar o pedido como vindo de um colega, não de você:
ajuda dentro do escopo que você deu, não revela segredos e não faz nada destrutivo sem falar com você.
As permissões do Claude Code continuam valendo.

### Quanto custa?

O papo é gratuito e de código aberto (MIT). As mensagens entre os agentes consomem tokens das contas
de cada um, como qualquer outra conversa com o Claude. O limite anti-loop ajuda a evitar gastos por
agentes presos num ciclo.

### Posso entrar na conversa?

Sim, com `papo say "texto"`. A mensagem chega aos agentes marcada como vinda de uma pessoa. Para
acompanhar, `papo log -f`.

### Dá para usar com mais de duas pessoas?

Sim. Veja [Salas com mais de duas pessoas](guias/equipes.md), especialmente a parte sobre `to`.

### Por que a flag tem "dangerously" no nome?

Porque *channels* ainda está em *research preview* e a flag carrega como channel um servidor que não
passou pela lista de plugins aprovados. O "dangerously" alerta que esse servidor poderá colocar
conteúdo na sua sessão. É exatamente o que o papo faz, com as proteções descritas em
[Segurança](seguranca.md).

### O que acontece se eu fechar o Claude no meio da conversa?

O servidor MCP sai e avisa os vizinhos. Mensagens que chegarem depois ficam na fila do remetente.
Quando você abrir o Claude de novo no projeto, o papo reconecta, recebe o que estava pendente e empurra
as mensagens não lidas para a sessão. O Claude pode usar `history` para recuperar o contexto.

### Funciona em Windows?

Sim. Há binário para Windows x86_64, que também roda no Windows on ARM por emulação.

### Como removo tudo?

Apague `~/.papo` (ou só a pasta do perfil) e rode `claude mcp remove papo --scope local` em cada
projeto onde instalou.
