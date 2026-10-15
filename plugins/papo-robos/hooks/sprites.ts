// Pixel art for the band: a little robot at a desk with a laptop. Drawn on a
// 32x20 pixel grid and packed two pixels per terminal cell with half blocks
// ("▀" top in the foreground, bottom in the background), so one scene is
// 32 columns by 10 rows.

export const WIDTH = 32
export const HEIGHT = 20
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
  deskFront: 0x6e4a30,
  deskLeg: 0x553823,
  lid: 0x2b2e35,
  screen: 0x0f1726,
  screenOff: 0x1a1d23,
  hinge: 0x7d838c,
  base: 0xc9ced6,
  baseShade: 0x9aa1ab,
  body: 0xeef3f8,
  bodyShade: 0xbac6d4,
  visor: 0x1d2535,
  eye: 0x5ce1e6,
  cheek: 0xff8fb1,
  stalk: 0x9aa1ab,
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
 * A deterministic line of "code" for screen row `n`: indentation and a few
 * colored tokens, the same every time so scrolling reads as a real file.
 */
function codeLine(n: number): { indent: number; tokens: { len: number; color: number }[] } {
  const h = (n * 2654435761) >>> 0
  const indent = pick([0, 1, 2, 2, 1, 3], h)
  const tokens: { len: number; color: number }[] = []
  let left = 9 - indent
  let k = h >>> 3
  while (left > 1) {
    const len = Math.min(left, 1 + (k % 4))
    tokens.push({ len, color: pick(CODE, k >>> 2) })
    left -= len + 1
    k = (k * 1103515245 + 12345) >>> 0
    if (k % 5 === 0) break
  }
  return { indent, tokens }
}

function desk(g: Grid) {
  rect(g, 0, 17, WIDTH, 1, C.deskTop)
  rect(g, 1, 18, WIDTH - 2, 1, C.deskFront)
  rect(g, 2, 19, 2, 1, C.deskLeg)
  rect(g, WIDTH - 4, 19, 2, 1, C.deskLeg)
}

function laptop(g: Grid, mood: Mood, frame: number) {
  // Lid with the screen facing the robot, hinge, and the keyboard base.
  rect(g, 18, 5, 12, 10, C.lid)
  rect(g, 18, 15, 12, 1, C.hinge)
  // The base reaches back under the robot's hands, so the keys are within reach.
  rect(g, 13, 16, 17, 1, C.base)
  put(g, 13, 16, C.baseShade)
  const sx = 19
  const sy = 6
  const sw = 10
  const sh = 8
  rect(g, sx, sy, sw, sh, mood === 'offline' ? C.screenOff : C.screen)
  if (mood === 'offline') return

  if (mood === 'done') {
    // Code at rest, dimmed, under a big check mark.
    for (let row = 0; row < sh; row++) {
      const { indent, tokens } = codeLine(row + 3)
      let x = sx + indent
      for (const t of tokens) {
        rect(g, x, sy + row, t.len, 1, 0x2c3a52)
        x += t.len + 1
      }
    }
    const check = [[21, 10], [22, 11], [23, 12], [24, 11], [25, 10], [26, 9], [27, 8]] as const
    for (const [x, y] of check) {
      put(g, x, y, C.check)
      put(g, x, y + 1, C.check)
    }
    return
  }

  // Working: the file scrolls up a line every other frame and the last line
  // grows as if being typed, with a blinking cursor at its end.
  const scroll = Math.floor(frame / 2)
  for (let row = 0; row < sh; row++) {
    const n = scroll + row
    const { indent, tokens } = codeLine(n)
    const typing = row === sh - 1
    let budget = typing ? 1 + (frame % 2) * 2 + (scroll % 3) : Infinity
    let x = sx + indent
    for (const t of tokens) {
      const len = Math.min(t.len, budget)
      if (len <= 0) break
      rect(g, x, sy + row, len, 1, t.color)
      budget -= len
      x += len + 1
      if (typing && budget <= 0) {
        if (frame % 2 === 0) put(g, Math.min(x - 1, sx + sw - 1), sy + row, C.cursor)
        break
      }
    }
  }
}

function robot(g: Grid, mood: Mood, frame: number) {
  const off = mood === 'offline'
  const body = off ? C.offBody : C.body
  const shade = off ? C.offShade : C.bodyShade

  // Antenna: busy blinks red, done glows green, offline is dark.
  put(g, 9, 2, C.stalk)
  const bulb = off ? C.offShade : mood === 'done' ? C.bulbDone : frame % 4 < 2 ? C.bulbBusy : C.bulbBusyDim
  rect(g, 8, 1, 3, 1, bulb)
  put(g, 9, 0, bulb)

  // Head and face.
  round(g, 4, 3, 10, 8, body)
  rect(g, 4, 4, 1, 6, shade)
  round(g, 6, 5, 7, 4, off ? C.offVisor : C.visor)
  const eye = off ? C.offEye : C.eye
  const blink = !off && frame % 24 === 23
  if (off || blink) {
    rect(g, 7, 7, 2, 1, eye)
    rect(g, 10, 7, 2, 1, eye)
  } else if (mood === 'done') {
    // Happy ^ ^ eyes.
    put(g, 7, 7, eye)
    put(g, 8, 6, eye)
    put(g, 9, 7, eye)
    put(g, 10, 7, eye)
    put(g, 11, 6, eye)
    put(g, 12, 7, eye)
  } else {
    // Looking right, at the screen.
    rect(g, 8, 6, 2, 2, eye)
    rect(g, 11, 6, 2, 2, eye)
  }
  if (!off) {
    put(g, 6, 9, C.cheek)
    put(g, 12, 9, C.cheek)
  }

  // Body, a chest light, and the stool it sits on behind the desk.
  round(g, 5, 11, 8, 6, body)
  rect(g, 5, 12, 1, 4, shade)
  put(g, 9, 13, off ? C.offShade : mood === 'done' ? C.bulbDone : C.eye)

  if (mood === 'working') {
    // Two arms hammering the keyboard in turns: one hand strikes (and its
    // key lights up) while the other is raised for the next key.
    const left = frame % 2 === 0
    arm(g, 12, 12, 16, left ? 11 : 14, shade, C.hand)
    arm(g, 12, 14, 13, left ? 14 : 11, body, C.hand)
    put(g, left ? 13 : 16, 16, C.key)
    put(g, left ? 14 : 17, 16, C.key)
  } else {
    // Arms resting at its sides.
    rect(g, 13, 12, 1, 4, shade)
  }
}

/**
 * A one-pixel arm from the shoulder to a 2x2 hand whose top-left is (hx, hy),
 * stepping one pixel at a time so diagonals stay connected.
 */
function arm(g: Grid, sx: number, sy: number, hx: number, hy: number, color: number, hand: number) {
  let x = sx
  let y = sy
  while (x !== hx || y !== hy) {
    put(g, x, y, color)
    if (x !== hx) x += Math.sign(hx - x)
    if (y !== hy) y += Math.sign(hy - y)
  }
  rect(g, hx, hy, 2, 2, hand)
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
