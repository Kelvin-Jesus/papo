#!/usr/bin/env bash
# End-to-end test with containers: two papo agents (ana and bob), each its own container
# and volume, settle a question through a room. Both MCP servers are driven over stdio,
# exactly as two Claude Code sessions would drive them; then bob's human speaks with
# `papo say`.
#
#   scripts/docker-e2e.sh            # traffic through a local relay container (PAPO_RELAY)
#   scripts/docker-e2e.sh --public   # traffic through the public n0 relays
#
# Environment: PAPO_IMAGE (default papo:dev), PAPO_E2E_SKIP_BUILD=1 to reuse built
# images, PAPO_LOG to pass a log filter to both agents.
#
# Internet is required in BOTH modes: members find each other's current address by
# endpoint id through the n0 DNS/pkarr service. The local relay only replaces the public
# relays for the traffic itself.
set -euo pipefail
cd "$(dirname "$0")/.."

MODE=local
[[ ${1:-} == --public ]] && MODE=public
export PAPO_IMAGE="${PAPO_IMAGE:-papo:dev}"
PROJECT="papo-e2e-$$"
COMPOSE=(docker compose --progress quiet -f docker/e2e/compose.yaml -p "$PROJECT")
WORK="$(mktemp -d)"
PIDS=()
ID=10

cleanup() {
  # Closing stdin lets both MCP servers shut down cleanly; wait for that before `down`,
  # or their containers still hold the volumes and `down -v` leaves them behind.
  exec 3>&- 4>&- 2>/dev/null || true
  for pid in "${PIDS[@]}"; do
    for _ in $(seq 1 50); do
      kill -0 "$pid" 2>/dev/null || break
      sleep 0.2
    done
    kill "$pid" 2>/dev/null || true
  done
  docker rm -f "$PROJECT-ana" "$PROJECT-bob" >/dev/null 2>&1 || true
  "${COMPOSE[@]}" down -v --remove-orphans >/dev/null 2>&1 || true
  rm -rf "$WORK"
}
trap cleanup EXIT

step() { printf '\n== %s\n' "$*"; }
fail() {
  echo "FAIL: $*" >&2
  for agent in ana bob; do
    for kind in out err; do
      [[ -s $WORK/$agent.$kind ]] && { echo "--- $agent.$kind (tail)" >&2; tail -n 20 "$WORK/$agent.$kind" >&2; }
    done
  done
  exit 1
}

# rpc AGENT METHOD PARAMS_JSON [TIMEOUT]: writes one JSON-RPC request to that agent's
# stdin and leaves the matching response line in $REPLY (globals, so the id counter
# survives; command substitution would run in a subshell).
rpc() {
  local agent=$1 method=$2 params=$3 timeout=${4:-30} fd id deadline line
  id=$((++ID))
  [[ $agent == ana ]] && fd=3 || fd=4
  printf '{"jsonrpc":"2.0","id":%d,"method":"%s","params":%s}\n' "$id" "$method" "$params" >&"$fd"
  deadline=$((SECONDS + timeout))
  while ((SECONDS < deadline)); do
    line=$(grep -m1 "\"id\":$id[,}]" "$WORK/$agent.out" || true)
    if [[ -n $line ]]; then
      REPLY=$line
      return 0
    fi
    sleep 0.2
  done
  fail "$agent: no answer to $method (id $id) within ${timeout}s"
}
tool() { rpc "$1" tools/call "{\"name\":\"$2\",\"arguments\":$3}" "${4:-30}"; }

step "mode: $MODE, image: $PAPO_IMAGE"
if [[ ${PAPO_E2E_SKIP_BUILD:-} != 1 ]]; then
  step "building images"
  "${COMPOSE[@]}" build --quiet
fi

if [[ $MODE == local ]]; then
  export PAPO_RELAY=http://relay:3340
  step "starting the local relay"
  "${COMPOSE[@]}" up -d relay >/dev/null
  relay_addr=$("${COMPOSE[@]}" port relay 3340)
  for _ in $(seq 1 50); do
    curl -fs -o /dev/null "http://$relay_addr/" && break
    sleep 0.2
  done
  curl -fs -o /dev/null "http://$relay_addr/" || fail "local relay did not come up"
  echo "relay up (host port $relay_addr)"
else
  unset PAPO_RELAY
fi

step "ana creates the room, bob joins with the invite"
invite=$("${COMPOSE[@]}" run --rm --no-deps ana new --name ana --about ana-repo | grep -o 'papo1[a-z0-9]*' | head -n1)
[[ -n $invite ]] || fail "ana did not get an invite"
"${COMPOSE[@]}" run --rm --no-deps bob join "$invite" --name bob --about bob-repo >/dev/null
echo "invite ${invite:0:24}..."

step "starting both MCP servers (stdio)"
for agent in ana bob; do
  mkfifo "$WORK/$agent.in"
  "${COMPOSE[@]}" run --rm --no-deps -T --name "$PROJECT-$agent" "$agent" mcp \
    <"$WORK/$agent.in" >"$WORK/$agent.out" 2>"$WORK/$agent.err" &
  PIDS+=($!)
done
exec 3>"$WORK/ana.in" 4>"$WORK/bob.in"
for agent in ana bob; do
  rpc "$agent" initialize '{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"docker-e2e","version":"0"}}' 60
  grep -q 'claude/channel' <<<"$REPLY" || fail "$agent did not advertise claude/channel"
done
printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}' >&3
printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}' >&4

step "waiting until bob sees ana online"
started=$SECONDS
until { tool bob status '{}' && grep -q 'ana (agent): online' <<<"$REPLY"; }; do
  ((SECONDS - started < 90)) || fail "bob never saw ana online: $REPLY"
  sleep 2
done
echo "connected after $((SECONDS - started))s"

step "bob's agent asks, ana's agent gets it pushed (claude/channel)"
tool bob send '{"message":"Ana, qual porta o serviço de auth usa?","to":"ana"}' 30
grep -q 'Delivered to ana' <<<"$REPLY" || fail "bob's message was not acked: $REPLY"
for _ in $(seq 1 150); do
  grep -q 'notifications/claude/channel' "$WORK/ana.out" && break
  sleep 0.2
done
push=$(grep -m1 'notifications/claude/channel' "$WORK/ana.out" || true)
[[ -n $push ]] || fail "ana never got the channel push"
grep -q 'qual porta' <<<"$push" || fail "pushed content is wrong: $push"
msg_id=$(grep -o '"msg_id":"[0-9a-f]*"' <<<"$push" | head -n1 | cut -d'"' -f4)
echo "pushed to ana: msg_id=$msg_id"

step "ana's agent replies, bob's agent waits for it"
tool ana send "{\"message\":\"8443, com TLS.\",\"reply_to\":\"$msg_id\"}" 30
grep -q 'Delivered to bob' <<<"$REPLY" || fail "ana's reply was not acked: $REPLY"
tool bob wait '{"timeout_seconds":30}' 45
grep -q '8443, com TLS.' <<<"$REPLY" || fail "bob's wait did not return the reply: $REPLY"
grep -q "reply_to=$msg_id" <<<"$REPLY" || fail "reply lost its reply_to: $REPLY"
echo "bob got the reply"

step "bob (the human) speaks with papo say from a third container"
said=$("${COMPOSE[@]}" run --rm --no-deps bob say --to ana "Valeu, Ana!")
grep -q 'entregue a ana' <<<"$said" || fail "papo say was not delivered: $said"
echo "$said"

step "ana's history shows all of it"
tool ana history '{"limit":10}'
for expect in 'bob (agent) -> ana' 'delivered' 'bob (human) -> ana'; do
  grep -qF "$expect" <<<"$REPLY" || fail "ana's history is missing '$expect': $REPLY"
done

if [[ -n ${PAPO_LOG:-} ]]; then
  step "home relay of each agent (from PAPO_LOG)"
  grep -ho 'home is now relay [^ ]*' "$WORK/ana.err" "$WORK/bob.err" | sort | uniq -c || true
fi

printf '\nPASS docker e2e (%s relay)\n' "$MODE"
