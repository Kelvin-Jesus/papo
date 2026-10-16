#!/usr/bin/env python3
"""Generates assets/robos/robos.svg and robos-en.svg: animated pixel robots.

Port of plugins/papo-robos/hooks/sprites.ts (the source of truth for the
sprites, palette and states). Pure SVG + CSS keyframes, no JS, no external
resources. Run: python3 scripts/robos-svg.py
"""
import os

W, H = 32, 20
S = 5  # screen px per sprite pixel
N = 16  # frames per cycle
DT = 0.25  # seconds per frame
CYCLE = N * DT

C = dict(deskTop=0x9c6b46, deskFront=0x6e4a30, deskLeg=0x553823, lid=0x2b2e35, screen=0x0f1726,
         screenOff=0x1a1d23, hinge=0x7d838c, base=0xc9ced6, baseShade=0x9aa1ab, body=0xeef3f8,
         bodyShade=0xbac6d4, visor=0x1d2535, eye=0x5ce1e6, cheek=0xff8fb1, stalk=0x9aa1ab,
         bulbBusy=0xff5c7a, bulbBusyDim=0x8f3245, bulbDone=0x7cd992, check=0x7cd992, hand=0xd3dde8,
         key=0x5ce1e6, cursor=0xe6edf3, offBody=0x6b7280, offShade=0x4b5260, offVisor=0x2a2f3a,
         offEye=0x5b6472)
CODE = [0xc678dd, 0x61afef, 0x98c379, 0xe5c07b, 0xe06c75, 0x56b6c2]


def sign(v): return (v > 0) - (v < 0)


def put(g, x, y, c):
    if 0 <= x < W and 0 <= y < H: g[y * W + x] = c


def rect(g, x, y, w, h, c):
    for j in range(y, y + h):
        for i in range(x, x + w): put(g, i, j, c)


def rnd(g, x, y, w, h, c):
    rect(g, x, y, w, h, c)
    for i, j in ((x, y), (x + w - 1, y), (x, y + h - 1), (x + w - 1, y + h - 1)): put(g, i, j, None)


def code_line(n):
    h = (n * 2654435761) & 0xFFFFFFFF
    indent = [0, 1, 2, 2, 1, 3][h % 6]
    tokens = []
    left = 9 - indent
    k = h >> 3
    while left > 1:
        ln = min(left, 1 + (k % 4))
        tokens.append((ln, CODE[(k >> 2) % 6]))
        left -= ln + 1
        k = int(float(k) * 1103515245 + 12345) % (2 ** 32)  # double math, like JS
        if k % 5 == 0: break
    return indent, tokens


def laptop(g, mood, frame):
    rect(g, 18, 5, 12, 10, C['lid']); rect(g, 18, 15, 12, 1, C['hinge'])
    rect(g, 13, 16, 17, 1, C['base']); put(g, 13, 16, C['baseShade'])
    sx, sy, sw, sh = 19, 6, 10, 8
    rect(g, sx, sy, sw, sh, C['screenOff'] if mood == 'offline' else C['screen'])
    if mood == 'offline': return
    if mood == 'done':
        for row in range(sh):
            indent, tokens = code_line(row + 3)
            x = sx + indent
            for ln, _ in tokens:
                rect(g, x, sy + row, ln, 1, 0x2c3a52); x += ln + 1
        for x, y in ((21, 10), (22, 11), (23, 12), (24, 11), (25, 10), (26, 9), (27, 8)):
            put(g, x, y, C['check']); put(g, x, y + 1, C['check'])
        return
    scroll = frame // 2
    for row in range(sh):
        indent, tokens = code_line(scroll + row)
        typing = row == sh - 1
        budget = 1 + (frame % 2) * 2 + (scroll % 3) if typing else 10 ** 9
        x = sx + indent
        for ln, col in tokens:
            l = min(ln, budget)
            if l <= 0: break
            rect(g, x, sy + row, l, 1, col)
            budget -= l; x += l + 1
            if typing and budget <= 0:
                if frame % 2 == 0: put(g, min(x - 1, sx + sw - 1), sy + row, C['cursor'])
                break


def arm(g, sx, sy, hx, hy, color, hand):
    x, y = sx, sy
    while x != hx or y != hy:
        put(g, x, y, color)
        if x != hx: x += sign(hx - x)
        if y != hy: y += sign(hy - y)
    rect(g, hx, hy, 2, 2, hand)


def robot(g, mood, frame):
    off = mood == 'offline'
    body = C['offBody'] if off else C['body']
    shade = C['offShade'] if off else C['bodyShade']
    put(g, 9, 2, C['stalk'])
    bulb = C['offShade'] if off else C['bulbDone'] if mood == 'done' else (C['bulbBusy'] if frame % 4 < 2 else C['bulbBusyDim'])
    rect(g, 8, 1, 3, 1, bulb); put(g, 9, 0, bulb)
    rnd(g, 4, 3, 10, 8, body); rect(g, 4, 4, 1, 6, shade)
    rnd(g, 6, 5, 7, 4, C['offVisor'] if off else C['visor'])
    eye = C['offEye'] if off else C['eye']
    blink = (not off) and frame % 24 == 23
    if off or blink:
        rect(g, 7, 7, 2, 1, eye); rect(g, 10, 7, 2, 1, eye)
    elif mood == 'done':
        for x, y in ((7, 7), (8, 6), (9, 7), (10, 7), (11, 6), (12, 7)): put(g, x, y, eye)
    else:
        rect(g, 8, 6, 2, 2, eye); rect(g, 11, 6, 2, 2, eye)
    if not off:
        put(g, 6, 9, C['cheek']); put(g, 12, 9, C['cheek'])
    rnd(g, 5, 11, 8, 6, body); rect(g, 5, 12, 1, 4, shade)
    put(g, 9, 13, C['offShade'] if off else C['bulbDone'] if mood == 'done' else C['eye'])
    if mood == 'working':
        left = frame % 2 == 0
        arm(g, 12, 12, 16, 11 if left else 14, shade, C['hand'])
        arm(g, 12, 14, 13, 14 if left else 11, body, C['hand'])
        put(g, 13 if left else 16, 16, C['key']); put(g, 14 if left else 17, 16, C['key'])
    else:
        rect(g, 13, 12, 1, 4, shade)


def scene(mood, frame):
    g = [None] * (W * H)
    rect(g, 0, 17, W, 1, C['deskTop']); rect(g, 1, 18, W - 2, 1, C['deskFront'])
    rect(g, 2, 19, 2, 1, C['deskLeg']); rect(g, W - 4, 19, 2, 1, C['deskLeg'])
    laptop(g, mood, frame); robot(g, mood, frame)
    return g


def rects(g, mask=None):
    """Pixel runs as one <path> per colour, in sprite pixel units (the parent scales)."""
    by = {}
    for y in range(H):
        x = 0
        while x < W:
            c = g[y * W + x]
            if c is None or (mask is not None and not mask[y * W + x]): x += 1; continue
            e = x
            while e + 1 < W and g[y * W + e + 1] == c and (mask is None or mask[y * W + e + 1]): e += 1
            by.setdefault(c, []).append(f'M{x} {y}h{e-x+1}v1h-{e-x+1}z')
            x = e + 1
    return ''.join(f'<path fill="#{c:06x}" d="{"".join(d)}"/>' for c, d in by.items())


def layers(grids):
    """Static layer (identical in all frames) plus per-frame differing pixels."""
    static = [all(g[i] == grids[0][i] for g in grids) for i in range(W * H)]
    return rects(grids[0], static), [rects(g, [not s for s in static]) for g in grids]


def frame_groups(frames, cls_prefix, step_frames):
    out = []
    for i, body in enumerate(frames):
        out.append(f'<g class="fr{" f0" if i == 0 else ""}" style="animation-delay:{i*DT-CYCLE:.2f}s">{body}</g>')
    return ''.join(out)


def build(labels):
    gw, gap, top = W * S, 56, 36
    pw = 3 * gw + 4 * gap
    ph = top + H * S + 50
    work = [scene('working', f) for f in range(N)]
    # robot 2: working for the first half of the cycle, then done
    w2 = work[:N // 2]
    st1, fr1 = layers(work)
    st2, fr2 = layers(w2)
    done = rects(scene('done', 0))
    off = rects(scene('offline', 0))
    css = (f'.fr{{visibility:hidden;animation:s {CYCLE:.2f}s steps(1,end) infinite}}'
           f'@keyframes s{{0%{{visibility:visible}}{100/N:.4f}%{{visibility:hidden}}100%{{visibility:hidden}}}}'
           f'.half{{animation:a {CYCLE:.2f}s steps(1,end) infinite}}.rest{{opacity:0;animation:b {CYCLE:.2f}s steps(1,end) infinite}}'
           '@keyframes a{0%{opacity:1}50%{opacity:0}100%{opacity:0}}'
           '@keyframes b{0%{opacity:0}50%{opacity:1}100%{opacity:1}}'
           'text{font:600 12px ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;fill:#c9d1d9}'
           '@media (prefers-reduced-motion:reduce){.fr,.half,.rest{animation:none}.fr.f0{visibility:visible}.half{opacity:1}.rest{opacity:0}}')
    # reduced motion: show only frame 0 of each working robot
    def robot_g(x, inner): return f'<g transform="translate({x},{top}) scale({S})">{inner}</g>'
    g1 = robot_g(gap, st1 + frame_groups(fr1, 'a', N))
    g2 = robot_g(2 * gap + gw, f'<g class="half">{st2}{half_frames(fr2)}</g><g class="rest">{done}</g>')
    g3 = robot_g(3 * gap + 2 * gw, off)
    ty = top + H * S + 30
    # A tuple label follows robot 2: the first text while it types, the second once it is done.
    def texto(i, t, cls=''):
        c = f' class="{cls}"' if cls else ''
        return f'<text{c} x="{gap + i*(gw+gap) + gw/2}" y="{ty}" text-anchor="middle">{t}</text>'
    texts = ''.join(texto(i, t[0], 'half') + texto(i, t[1], 'rest') if isinstance(t, tuple) else texto(i, t)
                    for i, t in enumerate(labels))
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {pw} {ph}" width="{pw}" height="{ph}" '
            f'shape-rendering="crispEdges" role="img" aria-label="{" / ".join(t[1] if isinstance(t, tuple) else t for t in labels)}">'
            f'<style>{css}</style><rect width="{pw}" height="{ph}" rx="14" fill="#0d1117"/>'
            f'{g1}{g2}{g3}{texts}</svg>')


def half_frames(frs):
    # 12 frames spread over the first half of the 24-frame cycle
    return ''.join(f'<g class="fr{" f0" if i == 0 else ""}" style="animation-delay:{i*DT-CYCLE:.2f}s">{b}</g>' for i, b in enumerate(frs))


if __name__ == '__main__':
    out = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'assets', 'robos')
    os.makedirs(out, exist_ok=True)
    for name, labels in (('robos.svg', ['seu agente — codando…', ('agente do colega — codando…', 'agente do colega — terminou ✓'), 'colega offline']),
                         ('robos-en.svg', ['your agent — coding…', ("colleague's agent — coding…", "colleague's agent — done ✓"), 'colleague offline'])):
        with open(os.path.join(out, name), 'w', encoding='utf-8') as f: f.write(build(labels))
        print(name, os.path.getsize(os.path.join(out, name)))
