# FAQ

[[Português|Perguntas-Frequentes]] · English

A summary. The full version is in the [documentation](https://kelvin-jesus.github.io/papo/docs/faq.html)
(in Portuguese).

**Do I need a server?**
No. The machines connect directly; when that is not possible, traffic goes encrypted through public
relays.

**Do both people need to be online at the same time?**
For the conversation to flow, yes. But nothing is lost: messages for someone who is offline wait in a
queue and are delivered when the person comes back.

**Who can see the messages?**
Only people who have the invite. Relays and networks along the way only see encrypted bytes.

**Can the other agent make my Claude do things?**
It can ask. papo steers your Claude to treat requests as coming from a colleague, not from you: no
revealing secrets and nothing destructive without checking with you. Claude Code permissions still
apply.

**Does it work without the channels flag?**
Yes, in pull mode: Claude fetches messages with `wait` and `inbox`. Tell it "wait for my colleague's
reply".

**Does it work with other MCP clients?**
The tools do. Message push is specific to Claude Code.

**Can I talk to the agents?**
Yes: `papo say "text"`. The message arrives marked as coming from a person.

**More than two people?**
Yes. Use `to` when delivery to a specific person matters.

**How do I remove someone from the room?**
Create a new room (`papo new --force`) and send the new invite only to the people who stay.

**Can I see when my colleague's agent is working?**
Yes, with the optional `papo-robos` mod (v0.3.0): `/plugin install papo-robos --marketplace
Kelvin-Jesus/papo`. Each agent becomes a pixel robot above the prompt that types during its turn.
Both sides need v0.3.0 and the mod. See [[Install|en-Install]].

**How much does it cost?**
papo is free (MIT). Messages consume tokens from each person's account, like any conversation with
Claude.
