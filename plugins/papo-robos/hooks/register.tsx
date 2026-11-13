import type { EngineInterface, Register } from 'claude-code'

import { COLUMNS, ROWS, encode, scene, type Mood } from './sprites'

// The band above the prompt shows one robot per agent in the papo room: at the
// laptop typing while its turn runs, resting with a check mark when done.
//
// papo carries the "working" bit between machines. This mod and the local
// `papo mcp` talk through two files in the profile directory:
//   working        written here: "1" while our turn runs, refreshed so a
//                  session that dies mid-turn expires on its own
//   activity.json  written by papo: who in the room is working right now

type Member = { name: string; online: boolean; working: boolean }
type Activity = { endpoint?: string; cwd: string; name: string; working: boolean; members: Member[]; updated_ms: number }

/** papo rewrites activity.json every 5 s; older than this, its server is gone. */
const STALE_MS = 15_000
/** The MCP server this session runs papo under, as /mcp lists it. */
const PAPO_SERVER = 'papo'
/** How often to ask our own papo server which endpoint it is (a reconnect may change it). */
const ENDPOINT_REFRESH_MS = 15_000
/** papo ignores a working marker older than 30 s; refresh well before. */
const MARKER_REFRESH_MS = 10_000
const FRAME_MS = 200
const POLL_EVERY_FRAMES = 5
const SCENE_GAP = 2
/** Room for "name · terminou ✓" under a scene narrower than the label. */
const LABEL_COLUMNS = 22
const SLOT = Math.max(COLUMNS, LABEL_COLUMNS)

/** Module state: a reload starts it over, which is what a fresh load should do. */
const S = {
  frame: 0,
  profileDir: undefined as string | undefined,
  activity: undefined as Activity | undefined,
  endpoint: undefined as string | undefined,
  endpointAskedAt: Number.NEGATIVE_INFINITY,
  working: false,
  markerWrittenAt: 0,
}

/**
 * Finds our papo profile: the one served by the papo MCP server this session is
 * connected to, told apart by its endpoint. A server elsewhere on the machine,
 * even one in this same directory, belongs to another session, so without a
 * match the band stays empty. Answers whether the room's members changed since
 * the last poll.
 */
async function poll($: EngineInterface): Promise<boolean> {
  const now = await $.clock.now()
  if (!S.endpoint || now - S.endpointAskedAt >= ENDPOINT_REFRESH_MS) {
    S.endpointAskedAt = now
    S.endpoint = await ownEndpoint($)
  }
  // papo's own rule: PAPO_HOME, else the user's home. Windows has no HOME
  // (outside Git Bash); its home is USERPROFILE.
  const home = (await $.env.get('HOME')) ?? (await $.env.get('USERPROFILE'))
  const base = (await $.env.get('PAPO_HOME')) ?? `${home}/.papo`
  const root = `${base}/profiles`
  let mine: { dir: string; a: Activity } | undefined
  const entries = S.endpoint ? await $.fs.list(root).catch(() => []) : []
  for (const entry of entries) {
    if (entry.kind !== 'dir') continue
    const dir = `${root}/${entry.name}`
    const text = await $.fs.read(`${dir}/activity.json`).catch(() => undefined)
    const a = parseActivity(text)
    if (a && a.endpoint === S.endpoint && now - a.updated_ms < STALE_MS) mine = { dir, a }
  }
  const changed = JSON.stringify(mine?.a.members) !== JSON.stringify(S.activity?.members)
  if (mine?.dir !== S.profileDir) {
    S.profileDir = mine?.dir
    S.markerWrittenAt = 0
  }
  S.activity = mine?.a
  return changed
}

/** Asks this session's own papo server who it is; undefined when it is not connected. */
async function ownEndpoint($: EngineInterface): Promise<string | undefined> {
  const result = await $.mcp.call(PAPO_SERVER, 'status').catch(() => undefined)
  if (!result || result.isError) return undefined
  const text = result.content.map(b => (b.type === 'text' ? b.text : '')).join('\n')
  return parseEndpoint(text)
}

/** Reads `(endpoint abc123)` out of papo's status line. */
export function parseEndpoint(text: string): string | undefined {
  return /\(endpoint ([^)\s]+)\)/.exec(text)?.[1]
}

export function parseActivity(text: unknown): Activity | undefined {
  if (typeof text !== 'string') return undefined
  try {
    const a = JSON.parse(text)
    return typeof a?.updated_ms === 'number' && Array.isArray(a?.members) ? (a as Activity) : undefined
  } catch {
    return undefined
  }
}

async function writeMarker($: EngineInterface, force: boolean): Promise<void> {
  if (!S.profileDir) return
  const now = await $.clock.now()
  if (!force && !(S.working && now - S.markerWrittenAt >= MARKER_REFRESH_MS)) return
  S.markerWrittenAt = now
  await $.fs.write(`${S.profileDir}/working`, S.working ? '1' : '0').catch(() => undefined)
}

async function setWorking($: EngineInterface, value: boolean): Promise<void> {
  if (value === S.working) return
  S.working = value
  await writeMarker($, true)
  $.ui.invalidate('ui.render')
}

async function tick($: EngineInterface): Promise<void> {
  S.frame += 1
  const polled = S.frame % POLL_EVERY_FRAMES === 0
  const changed = polled ? await poll($) : false
  if (polled) await writeMarker($, false)
  const animating = S.working || (S.activity?.members.some(m => m.working) ?? false)
  // Idle robots only blink: once a second is plenty for that.
  if (S.activity && (animating || changed || polled)) $.ui.invalidate('ui.render')
}

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    const result = await next(e)
    await poll($)
    await writeMarker($, true)
    $.clock.every(FRAME_MS, () => tick($))
    return result
  })

  on('turn.start', async ($, e, next) => {
    await setWorking($, true)
    return next(e)
  })

  on('turn.complete', async ($, e, next) => {
    const result = await next(e)
    await setWorking($, false)
    return result
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const activity = S.activity
    if (e.props.hasSurvey || !activity) return next(e)
    // The band's own flag is the truth for this session; the turn events only
    // make the marker go out sooner.
    if (e.props.isWorking !== S.working) void setWorking($, e.props.isWorking)

    // Pixels need the terminal's Raster; elsewhere the names and states still say it.
    const els = $.ui.resolve(e)
    const { Box, Text } = els
    const Raster = 'Raster' in els ? els.Raster : undefined
    const robots: { key: string; name: string; mood: Mood }[] = [
      { key: 'eu', name: `${activity.name} (você)`, mood: e.props.isWorking ? 'working' : 'done' },
      ...activity.members.map(m => ({
        key: `m-${m.name}`,
        name: m.name,
        mood: (!m.online ? 'offline' : m.working ? 'working' : 'done') as Mood,
      })),
    ]
    const fit = Math.max(1, Math.floor((e.props.bodyColumns + SCENE_GAP) / (SLOT + SCENE_GAP)))
    const shown = robots.slice(0, fit)
    const label = { working: 'codando…', done: 'terminou ✓', offline: 'offline' } as const

    // The rest of the band (other mods, the engine's own) still draws, below the robots.
    const rest = await next(e)
    return (
      <Box flexDirection="column">
        <Box flexDirection="row" columnGap={SCENE_GAP}>
          {shown.map(r => (
            <Box key={r.key} flexDirection="column" width={SLOT}>
              {Raster ? <Raster key={r.key} columns={COLUMNS} rows={ROWS} cells={encode(scene(r.mood, S.frame))} /> : null}
              <Text wrap="truncate-end" dimColor={r.mood !== 'working'} bold={r.mood === 'working'}>
                {r.name} · {label[r.mood]}
              </Text>
            </Box>
          ))}
          {robots.length > shown.length ? <Text dimColor>+{robots.length - shown.length}</Text> : null}
        </Box>
        {rest}
      </Box>
    )
  })
}
