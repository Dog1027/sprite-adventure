#!/usr/bin/env python3
"""產生精靈的占位動畫圖（像素風），輸出到 assets/sprites/<代號>.png。

用法：python3 tools/gen_placeholder_sprites.py

讀 data/species.toml 的每一種精靈，畫一隻 32×32、面向右邊的小生物，
再用位移、壓扁、變色做出各個動作。只用 Python 標準函式庫。

圖的排法（和 src/art.rs 一致）：每格 32×32，每一列是一種動作，由左到右播放
  第 1 列 待機 4 格（循環）
  第 2 列 攻擊 4 格
  第 3 列 技能 4 格
  第 4 列 受傷 2 格
  第 5 列 倒下 4 格（播完停在最後一格）
整張圖 128×160，沒用到的格子是透明的。

正式美術完成後，直接用同樣排法的 PNG 蓋掉這些檔案即可。
"""

import struct
import tomllib
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FRAME = 32
COLS = 4
# (動作, 格數)，順序就是列的順序
ROWS = [("idle", 4), ("attack", 4), ("skill", 4), ("hit", 2), ("defeated", 4)]
# 腳底所在的列（往下留 1 格透明）
FEET = 30

# 每種精靈的外觀：身體顏色、點綴色、身體半徑 (寬, 高)、特徵
LOOKS = {
    "ember_fox": ((232, 112, 48), (255, 214, 92), (9, 7), ["ears", "flame_tail"]),
    "pebble_turtle": ((112, 150, 96), (140, 118, 92), (11, 6), ["shell"]),
    "dew_sprout": ((150, 210, 120), (96, 180, 230), (7, 7), ["sprout"]),
    "gale_sparrow": ((150, 170, 200), (240, 170, 60), (7, 6), ["wing", "beak"]),
    "mud_newt": ((150, 110, 80), (230, 190, 120), (10, 5), ["tail"]),
    "thorn_beetle": ((90, 120, 70), (200, 190, 90), (9, 6), ["horn", "spikes"]),
    "frost_moth": ((200, 225, 245), (130, 180, 240), (6, 6), ["moth_wings", "antenna"]),
    "shadow_cat": ((70, 60, 100), (250, 220, 80), (8, 7), ["ears", "tail"]),
    "bell_jelly": ((210, 150, 220), (250, 230, 250), (8, 7), ["tentacles"]),
    "iron_boar": ((130, 130, 140), (235, 235, 220), (11, 7), ["tusk", "bristles"]),
    "thunder_lynx": ((230, 200, 80), (90, 150, 250), (9, 7), ["ears", "bolt", "tail"]),
    "moon_whale": ((70, 90, 160), (200, 220, 255), (12, 8), ["fin", "whale_tail"]),
}
DEFAULT_LOOK = ((180, 180, 180), (255, 255, 255), (8, 7), ["antenna"])

Color = tuple[int, int, int, int]
Image = list[list[Color]]
CLEAR: Color = (0, 0, 0, 0)


def blank(w: int = FRAME, h: int = FRAME) -> Image:
    return [[CLEAR] * w for _ in range(h)]


def shade(c, amount: float) -> Color:
    """amount > 0 變亮，< 0 變暗"""
    r, g, b = c[:3]
    if amount >= 0:
        r, g, b = (int(v + (255 - v) * amount) for v in (r, g, b))
    else:
        r, g, b = (int(v * (1 + amount)) for v in (r, g, b))
    return (r, g, b, 255)


def put(img: Image, x: int, y: int, c: Color) -> None:
    if 0 <= x < FRAME and 0 <= y < FRAME:
        img[y][x] = c


def ellipse(img, cx, cy, rx, ry, c) -> None:
    for y in range(cy - ry, cy + ry + 1):
        for x in range(cx - rx, cx + rx + 1):
            if ((x - cx) / (rx + 0.5)) ** 2 + ((y - cy) / (ry + 0.5)) ** 2 <= 1:
                put(img, x, y, c)


def triangle(img, a, b, c, color) -> None:
    def side(p, q, r):
        return (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])

    xs, ys = [a[0], b[0], c[0]], [a[1], b[1], c[1]]
    for y in range(min(ys), max(ys) + 1):
        for x in range(min(xs), max(xs) + 1):
            p = (x, y)
            d = (side(a, b, p), side(b, c, p), side(c, a, p))
            if all(v >= 0 for v in d) or all(v <= 0 for v in d):
                put(img, x, y, color)


def outline(img: Image, color: Color) -> Image:
    """在不透明像素外圍描一圈邊"""
    out = [row[:] for row in img]
    for y in range(FRAME):
        for x in range(FRAME):
            if img[y][x][3] == 0 and any(
                0 <= x + dx < FRAME and 0 <= y + dy < FRAME and img[y + dy][x + dx][3] > 0
                for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
            ):
                out[y][x] = color
    return out


def creature(species: str) -> Image:
    body, accent, (rx, ry), features = LOOKS.get(species, DEFAULT_LOOK)
    body, accent = shade(body, 0), shade(accent, 0)
    dark = shade(body, -0.45)
    img = blank()
    cx, cy = 15, FEET - ry

    # 身體後面的特徵
    for f in features:
        if f == "tail":
            for i in range(6):
                put(img, cx - rx - i, cy - i // 2, body)
                put(img, cx - rx - i, cy - i // 2 + 1, body)
        elif f == "flame_tail":
            triangle(img, (cx - rx + 1, cy), (cx - rx - 6, cy - 6), (cx - rx - 2, cy + 3), accent)
        elif f == "whale_tail":
            triangle(img, (cx - rx + 1, cy), (cx - rx - 4, cy - 5), (cx - rx - 4, cy + 3), body)
        elif f == "moth_wings":
            ellipse(img, cx - 3, cy - ry - 2, 5, 6, shade(accent, 0.3))
            ellipse(img, cx - 3, cy - ry - 2, 3, 4, accent)
        elif f == "tentacles":
            for i in range(-rx + 2, rx - 1, 3):
                for j in range(4):
                    put(img, cx + i + (j % 2), cy + ry - 1 + j, accent)

    ellipse(img, cx, cy, rx, ry, body)
    # 肚子
    ellipse(img, cx + 1, cy + ry // 2, max(rx - 4, 2), max(ry // 2 - 1, 1), shade(body, 0.35))

    # 身體上面的特徵
    head_x = cx + rx // 2
    top = cy - ry
    for f in features:
        if f == "ears":
            for ex in (head_x - 3, head_x + 2):
                triangle(img, (ex - 2, top + 2), (ex + 2, top + 2), (ex, top - 4), body)
                put(img, ex, top, accent)
        elif f == "shell":
            ellipse(img, cx - 1, top + 2, rx - 2, 4, accent)
            for sx in range(cx - rx + 3, cx + rx - 3, 3):
                put(img, sx, top + 1, shade(accent, -0.3))
        elif f == "sprout":
            put(img, head_x, top - 1, shade(accent, -0.2))
            put(img, head_x, top - 2, shade(accent, -0.2))
            ellipse(img, head_x - 2, top - 3, 2, 1, (110, 200, 90, 255))
            ellipse(img, head_x + 2, top - 4, 2, 1, (110, 200, 90, 255))
        elif f == "wing":
            ellipse(img, cx - 2, cy, 4, 3, shade(body, -0.25))
        elif f == "beak":
            triangle(img, (cx + rx, cy - 2), (cx + rx, cy + 1), (cx + rx + 3, cy - 1), accent)
        elif f == "horn":
            triangle(img, (cx + rx - 2, top + 3), (cx + rx, top + 4), (cx + rx + 4, top - 3), accent)
        elif f == "spikes":
            for sx in range(cx - rx + 2, cx + rx - 2, 3):
                triangle(img, (sx - 1, top + 1), (sx + 1, top + 1), (sx, top - 2), accent)
        elif f == "antenna":
            for i in range(4):
                put(img, head_x - 1 + i // 2, top - i, dark)
            put(img, head_x + 1, top - 4, accent)
        elif f == "tusk":
            put(img, cx + rx, cy + 1, accent)
            put(img, cx + rx + 1, cy, accent)
            put(img, cx + rx + 1, cy - 1, accent)
        elif f == "bristles":
            for sx in range(cx - rx + 3, cx + 2, 2):
                put(img, sx, top - 1, dark)
        elif f == "bolt":
            for i, (bx, by) in enumerate([(0, 0), (1, 1), (0, 2), (1, 3)]):
                put(img, cx - 3 + bx, top + 2 + by, accent)
        elif f == "fin":
            triangle(img, (cx - 3, top + 1), (cx + 2, top + 1), (cx - 2, top - 4), body)

    img = outline(img, dark)
    # 眼睛（面向右）
    ex, ey = head_x + 3, cy - ry // 2
    eye_white = (255, 255, 255, 255)
    pupil = (250, 220, 80, 255) if species == "shadow_cat" else (20, 20, 30, 255)
    put(img, ex, ey, eye_white)
    put(img, ex, ey + 1, eye_white)
    put(img, ex + 1, ey, pupil)
    put(img, ex + 1, ey + 1, pupil)
    return img


def transform(src: Image, dx=0, dy=0, squash=1.0, tint=None, tint_amount=0.0, alpha=1.0) -> Image:
    """位移、以腳底為基準垂直壓扁、混色、半透明"""
    out = blank()
    for y in range(FRAME):
        for x in range(FRAME):
            # 反推來源像素（最近點取樣）
            sy = round(FEET + (y - dy - FEET) / squash)
            sx = x - dx
            if not (0 <= sx < FRAME and 0 <= sy < FRAME):
                continue
            r, g, b, a = src[sy][sx]
            if a == 0:
                continue
            if tint is not None:
                r, g, b = (int(v + (t - v) * tint_amount) for v, t in zip((r, g, b), tint))
            out[y][x] = (r, g, b, int(a * alpha))
    return out


def sparkle(img: Image, x: int, y: int, c: Color) -> None:
    for dx, dy in ((0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)):
        put(img, x + dx, y + dy, c)


def frames(species: str) -> dict[str, list[Image]]:
    base = creature(species)
    accent = shade(LOOKS.get(species, DEFAULT_LOOK)[1], 0.2)
    white = (255, 255, 255)

    skill = []
    for i, (dx, amount) in enumerate([(0, 0.25), (0, 0.45), (3, 0.3), (1, 0.1)]):
        f = transform(base, dx=dx, tint=accent[:3], tint_amount=amount)
        if i in (1, 2):
            for sx, sy in ((4, 6), (26, 9), (8, 14) if i == 1 else (27, 18)):
                sparkle(f, sx, sy, accent)
        skill.append(f)

    return {
        "idle": [transform(base, dy=d) for d in (0, -1, -1, 0)],
        "attack": [transform(base, dx=x, dy=y) for x, y in ((-1, 0), (-2, 0), (4, -1), (2, 0))],
        "skill": skill,
        "hit": [
            transform(base, dx=-3, tint=white, tint_amount=0.75),
            transform(base, dx=-1, tint=(230, 60, 60), tint_amount=0.4),
        ],
        "defeated": [
            transform(base, squash=s, tint=(90, 90, 100), tint_amount=g, alpha=a)
            for s, g, a in ((0.85, 0.2, 1.0), (0.65, 0.45, 0.9), (0.45, 0.65, 0.8), (0.3, 0.8, 0.7))
        ],
    }


def write_png(path: Path, img: Image) -> None:
    h, w = len(img), len(img[0])
    raw = b"".join(b"\x00" + bytes(v for px in row for v in px) for row in img)

    def chunk(kind: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

    png = b"\x89PNG\r\n\x1a\n"
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
    png += chunk(b"IDAT", zlib.compress(raw, 9))
    png += chunk(b"IEND", b"")
    path.write_bytes(png)


def main() -> None:
    species = tomllib.loads((ROOT / "data/species.toml").read_text(encoding="utf-8"))
    out_dir = ROOT / "assets/sprites"
    out_dir.mkdir(parents=True, exist_ok=True)
    for sid in species:
        sheet = blank(FRAME * COLS, FRAME * len(ROWS))
        anims = frames(sid)
        for row, (name, count) in enumerate(ROWS):
            assert len(anims[name]) == count
            for col, f in enumerate(anims[name]):
                for y in range(FRAME):
                    for x in range(FRAME):
                        sheet[row * FRAME + y][col * FRAME + x] = f[y][x]
        write_png(out_dir / f"{sid}.png", sheet)
        print(f"assets/sprites/{sid}.png")


if __name__ == "__main__":
    main()
