#!/usr/bin/env python3
"""產生場景背景的占位圖（像素風、森林草原主題），輸出到 assets/scenes/<場景>.png。

用法：python3 tools/gen_placeholder_scenes.py

每張 225×400 像素，畫面上放大 2 倍剛好填滿直式虛擬畫面（450×800）。
背景刻意壓暗、降低彩度：介面文字與精靈直接畫在上面，要看得清楚。
只用 Python 標準函式庫；同樣的程式每次產生的圖都一樣。

場景與用到的畫面（和 src/art.rs 的 `Scene` 一致）：
  battle   戰鬥、擺位：草原中間踩平的空地，遠方是樹林
  map      冒險地圖：從空中俯瞰的森林與河流
  catch    捕捉：森林裡的空地
  shop     商店：帳篷與木頭地板
  rest     休息：夜晚的營火
  event    事件：起霧的森林小徑
  defeat   冒險失敗：黃昏
  victory  過關：日出

正式美術完成後，直接用同樣大小的 PNG 蓋掉這些檔案即可。
"""

import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from gen_placeholder_sprites import write_png  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
W, H = 225, 400

Rgb = tuple[int, int, int]


class Canvas:
    def __init__(self, seed: int):
        self.px: list[list[Rgb]] = [[(0, 0, 0)] * W for _ in range(H)]
        self.rng = random.Random(seed)

    def put(self, x: int, y: int, c: Rgb) -> None:
        if 0 <= x < W and 0 <= y < H:
            self.px[y][x] = c

    def get(self, x: int, y: int) -> Rgb:
        return self.px[min(max(y, 0), H - 1)][min(max(x, 0), W - 1)]

    def vgradient(self, y0: int, y1: int, top: Rgb, bottom: Rgb, dither: bool = True) -> None:
        """上下漸層；用有序抖色做出像素風的色階"""
        bayer = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]
        steps = 6
        for y in range(y0, y1):
            t = (y - y0) / max(y1 - y0 - 1, 1)
            for x in range(W):
                tt = t * steps
                if dither:
                    tt += (bayer[y % 4][x % 4] / 16 - 0.5)
                level = min(max(round(tt), 0), steps) / steps
                self.put(x, y, mix(top, bottom, level))

    def rect(self, x0: int, y0: int, x1: int, y1: int, c: Rgb) -> None:
        for y in range(y0, y1):
            for x in range(x0, x1):
                self.put(x, y, c)

    def ellipse(self, cx: float, cy: float, rx: float, ry: float, c: Rgb) -> None:
        for y in range(int(cy - ry) - 1, int(cy + ry) + 2):
            for x in range(int(cx - rx) - 1, int(cx + rx) + 2):
                if ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1:
                    self.put(x, y, c)

    def speckle(self, x0, y0, x1, y1, colors: list[Rgb], density: float) -> None:
        """隨機散點：草地、泥土的質感"""
        for y in range(y0, y1):
            for x in range(x0, x1):
                if self.rng.random() < density:
                    self.put(x, y, self.rng.choice(colors))

    def darken(self, amount_at) -> None:
        """依位置壓暗（amount_at(x, y) 回傳 0～1）"""
        for y in range(H):
            for x in range(W):
                a = amount_at(x, y)
                if a > 0:
                    self.px[y][x] = mix(self.px[y][x], (8, 10, 16), a)

    def save(self, name: str) -> None:
        out = ROOT / "assets/scenes" / f"{name}.png"
        out.parent.mkdir(parents=True, exist_ok=True)
        write_png(out, [[(*c, 255) for c in row] for row in self.px])
        print(f"assets/scenes/{name}.png")


def mix(a: Rgb, b: Rgb, t: float) -> Rgb:
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))  # type: ignore[return-value]


def shade(c: Rgb, amount: float) -> Rgb:
    return mix(c, (255, 255, 255), amount) if amount >= 0 else mix(c, (0, 0, 0), -amount)


# ── 共用配色 ──
SKY_TOP, SKY_LOW = (52, 78, 112), (104, 136, 150)
HILL = (58, 86, 84)
LEAF_DARK, LEAF, LEAF_LIGHT = (22, 44, 36), (34, 64, 46), (52, 88, 58)
TRUNK = (58, 42, 34)
GRASS, GRASS_DARK, GRASS_LIGHT = (50, 78, 52), (40, 64, 44), (66, 98, 62)
DIRT, DIRT_DARK, DIRT_LIGHT = (92, 84, 64), (76, 68, 52), (110, 100, 76)
FLOWERS = [(196, 186, 110), (190, 130, 150), (150, 170, 210)]


def vignette(cv: Canvas, strength: float = 0.45) -> None:
    def amount(x, y):
        dx, dy = (x - W / 2) / (W / 2), (y - H / 2) / (H / 2)
        return max(0.0, (dx * dx * 0.6 + dy * dy * 0.4) - 0.25) * strength

    cv.darken(amount)


def round_tree(cv: Canvas, x: int, base: int, size: int) -> None:
    cv.rect(x - 1, base - size // 2, x + 2, base, TRUNK)
    cy = base - size // 2 - size // 3
    cv.ellipse(x, cy, size * 0.6, size * 0.5, LEAF_DARK)
    cv.ellipse(x - 1, cy - 1, size * 0.5, size * 0.42, LEAF)
    cv.ellipse(x - size * 0.2, cy - size * 0.18, size * 0.22, size * 0.18, LEAF_LIGHT)


def pine(cv: Canvas, x: int, base: int, size: int, color: Rgb = LEAF_DARK) -> None:
    cv.rect(x, base - 3, x + 1, base, TRUNK)
    for i in range(size):
        half = (i * 0.45) + 1
        y = base - 3 - size + i
        for dx in range(-int(half), int(half) + 1):
            cv.put(x + dx, y, color if (dx + i) % 5 else shade(color, 0.12))


def treeline(cv: Canvas, y: int, count: int, size_range: tuple[int, int]) -> None:
    """一排遠近交錯的樹，`y` 是樹腳所在的高度"""
    xs = sorted(cv.rng.uniform(-8, W + 8) for _ in range(count))
    for x in xs:
        size = cv.rng.randint(*size_range)
        base = y + cv.rng.randint(-3, 3)
        if cv.rng.random() < 0.55:
            pine(cv, int(x), base, size + 6)
        else:
            round_tree(cv, int(x), base, size)


def grass_field(cv: Canvas, y0: int, y1: int) -> None:
    cv.vgradient(y0, y1, GRASS_LIGHT, GRASS_DARK)
    cv.speckle(0, y0, W, y1, [GRASS_DARK, GRASS_LIGHT, shade(GRASS, -0.15)], 0.18)
    # 草叢
    for _ in range((y1 - y0) * W // 260):
        x, y = cv.rng.randrange(W), cv.rng.randrange(y0, y1)
        c = shade(GRASS_LIGHT, 0.1)
        cv.put(x, y, c)
        cv.put(x - 1, y + 1, c)
        cv.put(x + 1, y + 1, c)
    # 小花
    for _ in range((y1 - y0) * W // 900):
        x, y = cv.rng.randrange(W), cv.rng.randrange(y0, y1)
        cv.put(x, y, cv.rng.choice(FLOWERS))


def rock(cv: Canvas, x: int, y: int, r: int) -> None:
    cv.ellipse(x, y, r + 0.5, r * 0.7, (70, 72, 74))
    cv.ellipse(x - 1, y - 1, r * 0.7, r * 0.45, (96, 98, 100))


def bush(cv: Canvas, x: int, y: int, r: int) -> None:
    cv.ellipse(x, y, r, r * 0.7, LEAF_DARK)
    cv.ellipse(x - 1, y - 1, r * 0.8, r * 0.55, LEAF)
    cv.put(x - r // 2, y - r // 2, LEAF_LIGHT)


# ── 各場景 ──


def battle() -> None:
    cv = Canvas(1)
    cv.vgradient(0, 70, SKY_TOP, SKY_LOW)
    # 遠山
    for x in range(W):
        h = 52 + int(6 * math.sin(x / 23) + 4 * math.sin(x / 9 + 1))
        for y in range(h, 74):
            cv.put(x, y, HILL)
    grass_field(cv, 62, H)
    treeline(cv, 66, 26, (10, 16))
    # 戰場：踩平的空地（橢圓），範圍約略涵蓋雙方棋盤（虛擬座標 120～600 → 圖上 60～300）
    cx, cy, rx, ry = 112, 182, 118, 128
    for y in range(40, 330):
        for x in range(W):
            d = ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2
            # 邊緣用亂數抖動，草地與泥土自然交錯
            if d < 1 - cv.rng.random() * 0.25:
                base = DIRT if (x * 7 + y * 13) % 11 else DIRT_DARK
                cv.put(x, y, mix(base, GRASS, max(0.0, d - 0.55)))
    cv.speckle(0, 60, W, 320, [DIRT_LIGHT, DIRT_DARK], 0.05)
    for _ in range(14):
        rock(cv, cv.rng.choice([cv.rng.randint(2, 18), cv.rng.randint(206, 222)]), cv.rng.randint(80, 300), 2)
    # 下方介面區：漸暗的土
    cv.darken(lambda x, y: min(1.0, max(0.0, (y - 300) / 70)) * 0.85)
    vignette(cv, 0.35)
    cv.save("battle")


def world_map() -> None:
    cv = Canvas(2)
    grass_field(cv, 0, H)
    # 河流：從上蜿蜒到下
    for y in range(H):
        x = 150 + 30 * math.sin(y / 40) + 10 * math.sin(y / 13)
        for dx in range(-5, 6):
            c = (52, 84, 112) if abs(dx) < 4 else (70, 104, 120)
            cv.put(int(x + dx), y, c)
        if y % 7 == 0:
            cv.put(int(x - 1), y, (96, 132, 150))
    # 森林叢
    for _ in range(40):
        x, y = cv.rng.randrange(W), cv.rng.randrange(20, H - 20)
        for _ in range(cv.rng.randint(3, 7)):
            pine(cv, x + cv.rng.randint(-10, 10), y + cv.rng.randint(-6, 6), cv.rng.randint(6, 10))
    for _ in range(25):
        rock(cv, cv.rng.randrange(W), cv.rng.randrange(H), cv.rng.randint(1, 3))
    # 地圖節點畫在上面，整張壓暗讓節點清楚
    cv.darken(lambda x, y: 0.45)
    vignette(cv, 0.6)
    cv.save("map")


def catch() -> None:
    cv = Canvas(3)
    cv.vgradient(0, 120, (30, 52, 52), (54, 82, 70))
    # 光線從樹葉縫隙灑下
    for i in range(5):
        x0 = 30 + i * 45
        for y in range(40, 330):
            for dx in range(6):
                x = x0 + dx + (y - 40) // 4
                if 0 <= x < W and (x + y) % 3:
                    cv.put(x, y, shade(cv.get(x, y), 0.06))
    grass_field(cv, 110, H)
    treeline(cv, 116, 22, (16, 26))
    # 兩側的大樹
    for x, size in ((8, 70), (218, 64)):
        cv.rect(x - 5, 60, x + 5, 360, TRUNK)
        cv.ellipse(x, 60, size * 0.7, size * 0.5, LEAF_DARK)
        cv.ellipse(x, 56, size * 0.55, size * 0.38, LEAF)
    for _ in range(10):
        bush(cv, cv.rng.randrange(W), cv.rng.randint(330, 390), cv.rng.randint(6, 12))
    cv.darken(lambda x, y: 0.25)
    vignette(cv, 0.5)
    cv.save("catch")


def shop() -> None:
    cv = Canvas(4)
    # 帳篷布：紅白條紋
    stripe_a, stripe_b = (104, 52, 50), (122, 108, 92)
    for y in range(0, 230):
        for x in range(W):
            c = stripe_a if (x // 16) % 2 == 0 else stripe_b
            # 往下越暗，像帳篷內部
            cv.put(x, y, shade(c, -0.25 - y / 600))
    # 帳篷邊緣的垂飾
    for x in range(0, W, 16):
        for i in range(8):
            for dx in range(16 - i * 2):
                cv.put(x + i + dx, 230 + i, shade(stripe_a if (x // 16) % 2 == 0 else stripe_b, -0.55))
    # 木頭地板
    for y in range(238, H):
        for x in range(W):
            plank = (y - 238) // 12
            seam = (x + plank * 37) % 64 == 0 or (y - 238) % 12 == 0
            c = (76, 56, 40) if not seam else (52, 38, 28)
            cv.put(x, y, shade(c, -0.1 if plank % 2 else 0.0))
    cv.speckle(0, 238, W, H, [(68, 50, 36), (84, 62, 44)], 0.08)
    # 掛燈
    for x in (40, 185):
        cv.rect(x, 0, x + 1, 40, (40, 34, 30))
        cv.ellipse(x, 46, 6, 7, (210, 170, 90))
        cv.ellipse(x, 46, 3, 4, (250, 220, 140))
    cv.darken(lambda x, y: 0.35)
    vignette(cv, 0.6)
    cv.save("shop")


def rest() -> None:
    cv = Canvas(5)
    cv.vgradient(0, 200, (12, 16, 34), (26, 36, 58))
    # 星星
    for _ in range(70):
        cv.put(cv.rng.randrange(W), cv.rng.randrange(0, 170), cv.rng.choice([(200, 210, 230), (140, 150, 180)]))
    cv.ellipse(196, 150, 9, 9, (220, 220, 200))
    cv.ellipse(200, 147, 8, 8, (16, 22, 42))
    grass_field(cv, 196, H)
    treeline(cv, 202, 26, (14, 22))
    cv.darken(lambda x, y: 0.5 if y > 196 else 0.0)
    # 營火的光（畫面中下方，介面卡片下面的空白處）
    fx, fy = 112, 300
    for y in range(H):
        for x in range(W):
            d = math.hypot((x - fx) / 1.4, y - fy)
            if d < 90:
                cv.put(x, y, mix(cv.get(x, y), (170, 110, 60), (1 - d / 90) ** 2 * 0.55))
    # 木柴與火焰
    for dx in range(-12, 13):
        cv.put(fx + dx, fy + 4 + abs(dx) // 6, (70, 46, 32))
        cv.put(fx + dx, fy + 5 + abs(dx) // 6, (50, 34, 26))
    cv.ellipse(fx, fy - 4, 7, 9, (220, 110, 40))
    cv.ellipse(fx, fy - 2, 4, 6, (250, 190, 80))
    cv.ellipse(fx, fy, 2, 3, (255, 240, 180))
    for x, y in ((fx - 26, fy + 6), (fx + 28, fy + 8)):
        cv.rect(x - 6, y, x + 6, y + 3, (64, 48, 40))
    cv.save("rest")


def event() -> None:
    cv = Canvas(6)
    cv.vgradient(0, 140, (40, 54, 60), (78, 92, 92))
    grass_field(cv, 130, H)
    treeline(cv, 140, 30, (14, 24))
    treeline(cv, 190, 12, (24, 34))
    # 小徑：往遠方收窄
    for y in range(140, H):
        t = (y - 140) / (H - 140)
        half = 6 + t * 50
        cx = 112 + 14 * math.sin(t * 3)
        for x in range(int(cx - half), int(cx + half)):
            cv.put(x, y, DIRT if (x + y) % 9 else DIRT_DARK)
    # 霧：越遠越白
    cv_mist = (120, 132, 132)
    for y in range(H):
        a = max(0.0, 1 - y / 260) * 0.55
        for x in range(W):
            cv.put(x, y, mix(cv.get(x, y), cv_mist, a))
    cv.darken(lambda x, y: 0.25)
    vignette(cv, 0.5)
    cv.save("event")


def ending(name: str, sky: tuple[Rgb, Rgb], sun: Rgb, dark: float, seed: int) -> None:
    cv = Canvas(seed)
    cv.vgradient(0, 250, *sky)
    cv.ellipse(112, 236, 30, 30, sun)
    for x in range(W):
        h = 226 + int(8 * math.sin(x / 19) + 5 * math.sin(x / 7 + 2))
        for y in range(h, 260):
            cv.put(x, y, mix(HILL, sky[1], 0.3))
    grass_field(cv, 250, H)
    treeline(cv, 256, 30, (12, 20))
    cv.darken(lambda x, y: dark * (0.4 + 0.6 * y / H))
    vignette(cv, 0.5)
    cv.save(name)


def main() -> None:
    battle()
    world_map()
    catch()
    shop()
    rest()
    event()
    ending("defeat", ((44, 26, 44), (150, 76, 60)), (200, 96, 60), 0.5, 7)
    ending("victory", ((70, 110, 160), (230, 190, 130)), (255, 230, 160), 0.2, 8)


if __name__ == "__main__":
    main()
