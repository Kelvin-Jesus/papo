import { expect, mock, test } from 'claude-code/testing'

import { COLUMNS, ROWS, base64, encode, scene } from './sprites'

const HOME = '/home/voce'
const PROFILES = `${HOME}/.papo/profiles`
const NOW = 1_000_000
const DEFAULT_PROFILE = { name: 'default', kind: 'dir', size: 0, mtimeMs: 0, isLink: false } as const

/** What `papo mcp` of profile `default` wrote: colega is mid-turn, outro is offline. */
const ACTIVITY = JSON.stringify({
  pid: 1,
  cwd: '/projeto',
  name: 'voce',
  working: false,
  members: [
    { name: 'colega', online: true, working: true },
    { name: 'outro', online: false, working: false },
  ],
  updated_ms: NOW - 1_000,
})

const band = (isWorking: boolean) => ({
  plugin: 'papo-robos',
  component: 'AbovePrompt' as const,
  props: {
    hasSurvey: false,
    isWorking,
    maxRows: 30,
    bodyColumns: 200,
    scroll: { offset: 0, bodyRows: 30 },
    view: {},
  } as never,
})

test('pixels pack two per cell into the Raster encoding', async () => {
  expect(base64(new Uint8Array([77, 97, 110]))).toBe('TWFu')
  expect(base64(new Uint8Array([77]))).toBe('TQ==')
  // 3 u32 words per cell, 4 bytes each, base64 grows by 4/3.
  const cells = encode(scene('working', 0))
  expect(cells.length).toBe(Math.ceil((COLUMNS * ROWS * 12) / 3) * 4)
  // Typing moves the arms and scrolls the code: consecutive frames differ.
  expect(encode(scene('working', 1))).not.toBe(cells)
  // A finished robot sits still apart from blinking.
  expect(encode(scene('done', 1))).toBe(encode(scene('done', 2)))
})

test('the band shows us and each room member, with what they are doing', async ($, on) => {
  const written: Record<string, string> = {}
  mock.env(on, { HOME })
  mock.clock(on, { now: NOW })
  // What the engine draws when the plugin yields the band: nothing.
  on('ui.render', async ($, e) => {
    const { Box } = $.ui.resolve(e)
    return <Box key="engine" />
  })
  on('session.cwd', async () => ({ value: '/projeto' }))
  on('fs.list', async (_$, e) => ({ value: e.path === PROFILES ? [DEFAULT_PROFILE] : [] }))
  on('fs.read', async (_$, e) => {
    if (e.path === `${PROFILES}/default/activity.json`) return { value: ACTIVITY }
    return { deny: 'ENOENT' }
  })
  on('fs.write', async (_$, e) => {
    written[e.path] = e.text
    return { value: undefined }
  })
  on('session.start', async (_$, e) => ({ cwd: e.cwd }))

  await $.session.start({ cwd: '/projeto', surface: 'terminal', isInteractive: true })
  expect(written[`${PROFILES}/default/working`]).toBe('0')

  const ui = await $.ui.mount({ ...band(true), surface: 'terminal' })
  expect((await ui.find({ type: 'Text', text: /voce \(você\) · codando/ }))?.type).toBe('Text')
  expect((await ui.find({ type: 'Text', text: /colega · codando/ }))?.type).toBe('Text')
  expect((await ui.find({ type: 'Text', text: /outro · offline/ }))?.type).toBe('Text')
  expect((await ui.find({ type: 'Raster', key: 'm-colega' }))?.type).toBe('Raster')
  // The band saw our turn running before any turn event: papo is told right away.
  expect(written[`${PROFILES}/default/working`]).toBe('1')
  await ui.unmount()

  // Elsewhere than the terminal there is no Raster, but the states still show.
  const desk = await $.ui.mount({ ...band(false), surface: 'desktop' })
  expect((await desk.find({ type: 'Text', text: /voce \(você\) · terminou/ }))?.type).toBe('Text')
  expect(await desk.find({ type: 'Raster' })).toBeUndefined()
  await desk.unmount()
})

test('without a live papo server for this session the band stays out of the way', async ($, on) => {
  mock.env(on, { HOME })
  mock.clock(on, { now: NOW })
  // What the engine draws when the plugin yields the band: nothing.
  on('ui.render', async ($, e) => {
    const { Box } = $.ui.resolve(e)
    return <Box key="engine" />
  })
  on('session.cwd', async () => ({ value: '/projeto' }))
  on('fs.list', async () => ({ value: [DEFAULT_PROFILE] }))
  // A file a crashed server left behind a minute ago.
  on('fs.read', async () => ({ value: JSON.stringify({ ...JSON.parse(ACTIVITY), updated_ms: NOW - 60_000 }) }))
  on('fs.write', async () => ({ value: undefined }))
  on('session.start', async (_$, e) => ({ cwd: e.cwd }))

  await $.session.start({ cwd: '/projeto', surface: 'terminal', isInteractive: true })
  const ui = await $.ui.mount({ ...band(true), surface: 'terminal' })
  expect(await ui.find({ type: 'Raster' })).toBeUndefined()
  expect(await ui.find({ type: 'Text', text: /codando/ })).toBeUndefined()
  await ui.unmount()
})
