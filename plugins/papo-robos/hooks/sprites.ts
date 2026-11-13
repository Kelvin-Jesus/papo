// Pixel art for the band: a little robot at a desk with a laptop. Drawn on a
// 16x10 pixel grid and packed two pixels per terminal cell with half blocks
// ("▀" top in the foreground, bottom in the background), so one scene is
// 16 columns by 5 rows: small enough to sit above the prompt unnoticed.

export const WIDTH = 16
export const HEIGHT = 10
export const COLUMNS = WIDTH
export const ROWS = HEIGHT / 2

/** Terminal default color, per the Raster cell encoding. */
const DEFAULT = 0x01000000
const UPPER_HALF = 0x2580 // ▀
const LOWER_HALF = 0x2584 // ▄
const SPACE = 0x20

export type Mood = 'working' | 'done' | 'offline'

type Px = number | null
/** Row-major pixels, `WIDTH * HEIGHT`; null is transparent. */
export type Grid = Px[]

const C = {
  deskTop: 0x9c6b46,
  deskLeg: 0x553823,
  lid: 0x2b2e35,
  screen: 0x0f1726,
  screenOff: 0x1a1d23,
  base: 0xc9ced6,
  baseShade: 0x9aa1ab,
  body: 0xeef3f8,
  bodyShade: 0xbac6d4,
  visor: 0x1d2535,
  eye: 0x5ce1e6,
  bulbBusy: 0xff5c7a,
  bulbBusyDim: 0x8f3245,
  bulbDone: 0x7cd992,
  check: 0x7cd992,
  hand: 0xd3dde8,
  key: 0x5ce1e6,
  cursor: 0xe6edf3,
  offBody: 0x6b7280,
  offShade: 0x4b5260,
  offVisor: 0x2a2f3a,
  offEye: 0x5b6472,
}

const CODE = [0xc678dd, 0x61afef, 0x98c379, 0xe5c07b, 0xe06c75, 0x56b6c2] as const

function grid(): Grid {
  return Array<Px>(WIDTH * HEIGHT).fill(null)
}

function put(g: Grid, x: number, y: number, c: Px) {
  if (x >= 0 && x < WIDTH && y >= 0 && y < HEIGHT) g[y * WIDTH + x] = c
}

export function pixel(g: Grid, x: number, y: number): Px {
  return g[y * WIDTH + x] ?? null
}

/** Picks from a non-empty list, wrapping the index. */
function pick<T>(list: readonly [T, ...T[]], i: number): T {
  return list[i % list.length] ?? list[0]
}

function rect(g: Grid, x: number, y: number, w: number, h: number, c: Px) {
  for (let j = y; j < y + h; j++) for (let i = x; i < x + w; i++) put(g, i, j, c)
}

/** A rect with its four corner pixels left out: reads as rounded at this size. */
function round(g: Grid, x: number, y: number, w: number, h: number, c: Px) {
  rect(g, x, y, w, h, c)
  for (const [i, j] of [[x, y], [x + w - 1, y], [x, y + h - 1], [x + w - 1, y + h - 1]] as const) put(g, i, j, null)
}

/**
 * A deterministic line of "code" for screen row `n`, at most `width` pixels:
 * indentation and a few colored tokens, the same every time so scrolling reads
 * as a real file.
 */
function codeLine(n: number, width: number): { indent: number; tokens: { len: number; color: number }[] } {
  const h = (n * 2654435761) >>> 0
  const indent = pick([0, 1, 1, 0], h)
  const tokens: { len: number; color: number }[] = []
  let left = width - indent
  let k = h >>> 3
  while (left > 0) {
    const len = Math.min(left, 1 + (k % 2))
    tokens.push({ len, color: pick(CODE, k >>> 2) })
    left -= len + 1
    k = (k * 1103515245 + 12345) >>> 0
    if (k % 3 === 0) break
  }
  return { indent, tokens }
}

function desk(g: Grid) {
  rect(g, 0, 8, WIDTH, 1, C.deskTop)
  put(g, 1, 9, C.deskLeg)
  put(g, WIDTH - 2, 9, C.deskLeg)
}

const SX = 10
const SY = 3
const SW = 4
const SH = 3

function laptop(g: Grid, mood: Mood, frame: number) {
  // Lid with the screen facing the robot, and the keyboard base reaching back
  // under the robot's hands.
  rect(g, 9, 2, 6, 5, C.lid)
  rect(g, 7, 7, 8, 1, C.base)
  put(g, 7, 7, C.baseShade)
  rect(g, SX, SY, SW, SH, mood === 'offline' ? C.screenOff : C.screen)
  if (mood === 'offline') return

  if (mood === 'done') {
    // A check mark across the screen.
    for (const [x, y] of [[SX, SY + 1], [SX + 1, SY + 2], [SX + 2, SY + 1], [SX + 3, SY]] as const) put(g, x, y, C.check)
    return
  }

  // Working: the file scrolls up a line every other frame and the last line
  // grows as if being typed, with a blinking cursor at its end.
  const scroll = Math.floor(frame / 2)
  for (let row = 0; row < SH; row++) {
    const { indent, tokens } = codeLine(scroll + row, SW)
    const typing = row === SH - 1
    let budget = typing ? 1 + (frame % 2) : Infinity
    let x = SX + indent
    for (const t of tokens) {
      const len = Math.min(t.len, budget)
      if (len <= 0) break
      rect(g, x, SY + row, len, 1, t.color)
      budget -= len
      x += len + 1
      if (typing && budget <= 0) {
        if (frame % 2 === 0) put(g, Math.min(x - 1, SX + SW - 1), SY + row, C.cursor)
        break
      }
    }
  }
}

function robot(g: Grid, mood: Mood, frame: number) {
  const off = mood === 'offline'
  const body = off ? C.offBody : C.body
  const shade = off ? C.offShade : C.bodyShade

  // Antenna bulb: busy blinks red, done glows green, offline is dark.
  const bulb = off ? C.offShade : mood === 'done' ? C.bulbDone : frame % 4 < 2 ? C.bulbBusy : C.bulbBusyDim
  put(g, 4, 0, bulb)

  // Head with a visor; the eyes look right, at the screen.
  round(g, 1, 1, 6, 4, body)
  rect(g, 1, 2, 1, 2, shade)
  rect(g, 2, 2, 4, 2, off ? C.offVisor : C.visor)
  const eye = off ? C.offEye : C.eye
  const blink = !off && frame % 24 === 23
  if (off || blink) {
    put(g, 3, 3, eye)
    put(g, 5, 3, eye)
  } else if (mood === 'done') {
    // Content, half-closed eyes.
    put(g, 3, 2, eye)
    put(g, 5, 2, eye)
  } else {
    rect(g, 3, 2, 1, 2, eye)
    rect(g, 5, 2, 1, 2, eye)
  }

  // Body with a chest light.
  rect(g, 2, 5, 4, 3, body)
  put(g, 2, 6, shade)
  put(g, 4, 6, off ? C.offShade : mood === 'done' ? C.bulbDone : C.eye)

  if (mood === 'working') {
    // Two hands hammering the keyboard in turns: one strikes while the other
    // is raised for the next key.
    const left = frame % 2 === 0
    put(g, 6, 6, shade)
    put(g, 7, left ? 7 : 6, C.hand)
    put(g, 8, left ? 6 : 7, C.hand)
    put(g, left ? 9 : 10, 7, C.key)
  } else {
    // Arm resting at its side.
    put(g, 6, 6, shade)
    put(g, 6, 7, shade)
  }
}

/** Pixels of one scene at animation `frame`. */
export function scene(mood: Mood, frame: number): Grid {
  const g = grid()
  desk(g)
  laptop(g, mood, frame)
  robot(g, mood, frame)
  return g
}

/** The Raster `cells` for a pixel grid: base64 of `[codePoint, fg, bg]` u32 triplets. */
export function encode(g: Grid): string {
  const words = new Uint32Array(COLUMNS * ROWS * 3)
  let i = 0
  for (let r = 0; r < ROWS; r++) {
    for (let x = 0; x < COLUMNS; x++) {
      const top = pixel(g, x, 2 * r)
      const bottom = pixel(g, x, 2 * r + 1)
      if (top === null && bottom === null) words.set([SPACE, DEFAULT, DEFAULT], i)
      else if (top === null) words.set([LOWER_HALF, bottom ?? DEFAULT, DEFAULT], i)
      else words.set([UPPER_HALF, top, bottom ?? DEFAULT], i)
      i += 3
    }
  }
  return base64(new Uint8Array(words.buffer))
}

const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'

/** Standard padded base64; written out so it does not depend on the runtime having toBase64. */
export function base64(bytes: Uint8Array): string {
  let out = ''
  for (let i = 0; i < bytes.length; i += 3) {
    const a = bytes[i] ?? 0
    const b = bytes[i + 1] ?? 0
    const c = bytes[i + 2] ?? 0
    const n = (a << 16) | (b << 8) | c
    out += ALPHABET[(n >> 18) & 63]! + ALPHABET[(n >> 12) & 63]!
    out += i + 1 < bytes.length ? ALPHABET[(n >> 6) & 63]! : '='
    out += i + 2 < bytes.length ? ALPHABET[n & 63]! : '='
  }
  return out
}
