#!/usr/bin/env bash
# Two real Claude Code sessions talk through papo, end to end (pull mode: send/wait).
#
# Uses your Claude Code login and costs a few cents per run (about US$ 0.11 with sonnet on
# 2026-10-02). It needs the internet (Claude API, n0 DNS/relays). Channels push is not covered:
# `--dangerously-load-development-channels` asks for an interactive confirmation.
#
#   scripts/validate-claude-code.sh                 # uses target/debug/papo (built if missing)
#   PAPO_BIN=$(command -v papo) CLAUDE_MODEL=opus scripts/validate-claude-code.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
bin="${PAPO_BIN:-$root/target/debug/papo}"
model="${CLAUDE_MODEL:-sonnet}"
if [[ ! -x "$bin" ]]; then
  (cd "$root" && cargo build --locked)
fi
command -v claude >/dev/null || { echo "claude (Claude Code CLI) is not on PATH" >&2; exit 2; }

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work"/{home-voce,home-colega,voce-proj,colega-proj}

invite="$(PAPO_HOME="$work/home-voce" "$bin" new --name voce --about api-pagamentos | grep -o 'papo1[a-z0-9]*')"
PAPO_HOME="$work/home-colega" "$bin" join "$invite" --name colega --about notificacoes >/dev/null
for who in voce colega; do
  printf '{"mcpServers":{"papo":{"command":"%s","args":["mcp"],"env":{"PAPO_HOME":"%s"}}}}\n' \
    "$bin" "$work/home-$who" >"$work/mcp-$who.json"
done

tools="mcp__papo__send,mcp__papo__wait,mcp__papo__inbox,mcp__papo__status,mcp__papo__history"
run_claude() { # $1 = profile, $2 = prompt
  (cd "$work/$1-proj" && timeout 600 claude -p --model "$model" --strict-mcp-config \
    --mcp-config "$work/mcp-$1.json" --setting-sources project --allowedTools "$tools" \
    --no-session-persistence --output-format json "$2")
}

run_claude colega "Você é o agente do colega, trabalhando no projeto notificacoes. Use a ferramenta wait do papo com timeout_seconds 300 para esperar uma mensagem de outro agente. Quando chegar, responda com a ferramenta send passando reply_to, usando estes fatos do seu projeto: o handler de webhooks valida o header X-Signature-256 no formato sha256=<hex> (HMAC-SHA256 do corpo) e espera o campo type, não event. Depois de responder, chame wait de novo com timeout_seconds 180; se chegar um resumo final, não responda a ele e termine dizendo o que ficou combinado." \
  >"$work/colega.json" &
colega_pid=$!
sleep 10
run_claude voce "Você é o meu agente no projeto api-pagamentos. Vou emitir um webhook payment.confirmed para o projeto notificacoes do meu colega. Pergunte ao agente do colega, pelo papo, qual header de assinatura e qual nome de campo de tipo o handler dele espera. Use send e depois wait (timeout_seconds 300). Quando tiver a resposta, mande uma mensagem final curta resumindo o combinado e me diga o resultado." \
  >"$work/voce.json"
wait "$colega_pid"

echo "== conversa (papo log do lado voce)"
PAPO_HOME="$work/home-voce" "$bin" log -n 20
log_voce="$(PAPO_HOME="$work/home-voce" "$bin" log -n 50)"
log_colega="$(PAPO_HOME="$work/home-colega" "$bin" log -n 50)"

fail=0
check() { if grep -q -- "$2" <<<"$1"; then echo "ok   $3"; else echo "FAIL $3"; fail=1; fi; }
check "$log_voce" "delivered .* to colega" "a pergunta foi entregue ao colega"
check "$log_colega" "reply to " "o agente do colega respondeu com reply_to"
check "$log_voce" "X-Signature-256" "a resposta trouxe o header certo"
check "$log_voce" "type" "a resposta trouxe o campo de tipo"
replies_from_colega="$(grep -c "colega (agent) -> voce" <<<"$log_voce" || true)"
if [[ "$replies_from_colega" -eq 1 ]]; then echo "ok   o colega não respondeu ao resumo final (sem loop)"; else echo "FAIL o colega mandou $replies_from_colega mensagens (esperado: 1)"; fail=1; fi
python3 - "$work/voce.json" "$work/colega.json" <<'EOF'
import json, sys
total = 0.0
for path in sys.argv[1:]:
    data = json.load(open(path))
    total += data.get("total_cost_usd") or 0
    print(f"     {path.split('/')[-1]}: {data.get('subtype')}, {data.get('num_turns')} turnos")
print(f"     custo total: US$ {total:.4f}")
EOF
exit "$fail"
