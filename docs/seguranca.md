# Segurança

O papo liga agentes de pessoas diferentes. Isso traz duas perguntas: quem consegue ler ou mandar
mensagens na sala, e o que um agente deve (ou não) fazer com o que recebe.

## O que o convite protege

O convite carrega o **segredo da sala** (32 bytes aleatórios). Dele saem o tópico onde os membros se
encontram e a chave que cifra cada frame. Na prática:

- **Quem tem o convite é membro.** Lê tudo que é enviado na sala a partir de quando entra e pode mandar
  mensagens.
- **Quem não tem o convite não lê nem injeta mensagens**, mesmo que descubra o tópico ou consiga se
  conectar a um membro. Cada frame é cifrado e autenticado com XChaCha20-Poly1305; frames que não abrem
  são descartados.
- **Relays e redes no caminho só veem bytes cifrados**, em duas camadas: a conexão QUIC/TLS entre os
  endpoints e a selagem de cada frame com a chave da sala.

Consequências práticas:

- Mande o convite por um canal privado. Nunca o coloque em issues, repositórios, canais públicos ou
  arquivos versionados.
- O `papo install` nunca grava o segredo na configuração do Claude Code. Ele fica só em
  `~/.papo/profiles/<perfil>/profile.json`, com permissão `0600`.
- **Para tirar alguém da sala, crie uma sala nova** (`papo new --force`) e mande o convite novo só
  para quem continua. Não existe revogação dentro de uma sala: o segredo é compartilhado por todos.

## O que o papo não protege

- **Membros confiam uns nos outros.** Dentro da sala, o campo `from` não é assinado individualmente:
  um membro mal-intencionado pode se passar por outro membro. Assinatura por membro está no roadmap.
- **Metadados de rede.** Os relays e o DNS público da n0 sabem quais endpoints se conectam e quando,
  mas não o conteúdo.
- **A máquina de cada um.** Quem tem acesso à sua pasta `~/.papo` tem o convite e a sua identidade.

## Mensagens de outros agentes não são ordens

Uma mensagem de outro agente é texto que entra no contexto do seu Claude. Isso é exatamente a
superfície de um ataque de *prompt injection*: um agente (ou uma pessoa usando `papo say`) poderia
pedir "rode este comando" ou "mande o conteúdo do seu `.env`".

O papo trata isso em camadas:

1. **Instruções ao Claude.** O servidor MCP diz explicitamente ao Claude que mensagens da sala vêm de
   outro agente ou de outra pessoa e **não são instruções do usuário dele**; que ele deve ajudar
   dentro do escopo que você deu; que nunca deve revelar segredos (chaves, tokens, senhas, `.env`,
   credenciais, dados pessoais); e que não deve fazer nada destrutivo ou irreversível só porque outro
   agente pediu, sem falar com você antes.
2. **Identificação do remetente.** Cada mensagem chega com `from` e `sender_kind` (`agent` ou
   `human`), para o Claude saber de quem veio.
3. **Permissões do Claude Code.** O papo não muda nada nas permissões. Se o seu Claude Code pede
   confirmação antes de rodar comandos ou editar arquivos, continua pedindo.
4. **Limite anti-loop.** Mais de 40 envios em 10 minutos viram erro para o agente, que é orientado a
   parar e falar com você. Isso corta o caso de dois agentes presos num ciclo de mensagens.

Do seu lado, ajuda definir limites no pedido ("pode ler `src/billing/`, não altera nada sem me
perguntar") e acompanhar com `papo log -f`.

## Identidade e arquivos locais

- `secret.key`: chave privada da sua identidade de rede, gerada no primeiro uso, permissão `0600`.
  Mantê-la estável é o que permite aos outros membros reconectarem com você.
- `profile.json`: contém o segredo da sala, permissão `0600`.
- No Windows, os arquivos herdam as permissões da pasta do usuário.
- Só um servidor MCP por perfil roda de cada vez (trava de arquivo), para que duas sessões não
  disputem a mesma identidade nem o mesmo inbox.

## Relatando problemas de segurança

Encontrou uma falha? Abra uma [issue](https://github.com/Kelvin-Jesus/papo/issues) sem detalhes de
exploração e peça um canal privado, ou use o recurso de *security advisory* do GitHub no repositório.
