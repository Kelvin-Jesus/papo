#!/usr/bin/env bash
# Smoke test for the papo container image: the CLI, a persistent named volume, and the
# MCP server over stdio the way Claude Code drives it (`docker run -i ... mcp`).
#
#   scripts/docker-smoke.sh [image]        # default image: papo:dev
#
# Needs no internet: the MCP server binds its endpoint but no peer is involved.
set -euo pipefail

IMAGE="${1:-papo:dev}"
VOL="papo-smoke-$$"
HOLDER="papo-smoke-holder-$$"

cleanup() {
  docker rm -f "$HOLDER" >/dev/null 2>&1 || true
  docker volume rm -f "$VOL" >/dev/null 2>&1 || true
}
trap cleanup EXIT

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

docker volume create "$VOL" >/dev/null

version=$(docker run --rm "$IMAGE" --version)
[[ $version == papo\ * ]] || fail "unexpected --version output: $version"
echo "ok   --version -> $version"

out=$(docker run --rm -v "$VOL:/data" "$IMAGE" new --name smoke --about smoke-test)
grep -q 'papo1' <<<"$out" || fail "new did not print an invite"
echo "ok   new -> room created, invite printed, profile stored in the volume"

out=$(docker run --rm -v "$VOL:/data" "$IMAGE" install --print)
grep -q '"mcp"' <<<"$out" || fail "install --print did not print an MCP entry"
echo "ok   install --print"

# MCP over stdio. stdin stays open for a few seconds so the server can bind its endpoint
# and answer the status call before it sees EOF and shuts down.
mcp_session() {
  {
    printf '%s\n' \
      '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}' \
      '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
      '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
      '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"status","arguments":{}}}'
    sleep "${1:-8}"
  } | docker run -i --rm -v "$VOL:/data" "$IMAGE" mcp
}

out=$(mcp_session 8)
grep '"id":1' <<<"$out" | grep -q 'claude/channel' || fail "initialize did not advertise claude/channel: $out"
echo "ok   mcp initialize -> claude/channel capability advertised"
for tool in send wait inbox history status; do
  grep '"id":2' <<<"$out" | grep -q "\"name\":\"$tool\"" || fail "tools/list is missing $tool"
done
echo "ok   mcp tools/list -> send, wait, inbox, history, status"
grep '"id":3' <<<"$out" | grep -q 'You are \\"smoke\\"' || fail "status did not answer for profile smoke: $out"
echo "ok   mcp status -> answers for the profile stored in the volume"

# One MCP server per profile: a second container on the same volume must be refused,
# because two processes would fight over the same endpoint identity.
docker run -d -i --name "$HOLDER" -v "$VOL:/data" "$IMAGE" mcp >/dev/null
sleep 3
out=$(mcp_session 4)
grep '"id":3' <<<"$out" | grep -q 'already running' || fail "second server on the same profile was not refused: $out"
echo "ok   profile lock -> a second container on the same volume is refused"

echo "PASS docker smoke test ($IMAGE)"
