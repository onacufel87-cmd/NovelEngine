#!/usr/bin/env python3
"""将 logo 背景转为透明 PNG，裁剪留白并放大到标准图标画布。"""

from __future__ import annotations

import sys
from collections import deque
from pathlib import Path

from PIL import Image


def _alpha_stats(img: Image.Image) -> tuple[float, int, int]:
    rgba = img.convert("RGBA")
    alpha = list(rgba.split()[3].getdata())
    total = len(alpha)
    transparent = sum(1 for a in alpha if a < 16)
    return transparent / total, min(alpha), max(alpha)


def is_flat_light_bg(r: int, g: int, b: int) -> bool:
    """白底 / 灰白棋盘格 / 浅灰底（Gemini 假透明图常见）。"""
    spread = max(r, g, b) - min(r, g, b)
    if spread > 14:
        return False
    avg = (r + g + b) / 3
    return avg >= 185


def flood_remove_light_background(img: Image.Image) -> Image.Image:
    """从四边泛洪，仅去掉与边缘连通的浅色背景，保留 logo 内部白/浅色。"""
    rgba = img.convert("RGBA")
    pixels = rgba.load()
    w, h = rgba.size
    visited = [[False] * w for _ in range(h)]
    queue: deque[tuple[int, int]] = deque()

    def try_seed(x: int, y: int) -> None:
        if visited[y][x]:
            return
        r, g, b, _ = pixels[x, y]
        if is_flat_light_bg(r, g, b):
            visited[y][x] = True
            queue.append((x, y))

    for x in range(w):
        try_seed(x, 0)
        try_seed(x, h - 1)
    for y in range(h):
        try_seed(0, y)
        try_seed(w - 1, y)

    while queue:
        x, y = queue.popleft()
        r, g, b, _ = pixels[x, y]
        pixels[x, y] = (r, g, b, 0)
        for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
            if 0 <= nx < w and 0 <= ny < h and not visited[ny][nx]:
                nr, ng, nb, _ = pixels[nx, ny]
                if is_flat_light_bg(nr, ng, nb):
                    visited[ny][nx] = True
                    queue.append((nx, ny))

    return rgba


def remove_light_background(
    img: Image.Image,
    *,
    threshold: int = 185,
    feather: int = 10,
) -> Image.Image:
    """兜底：剩余孤立浅色像素（仅高亮且无色彩倾向）。"""
    rgba = img.convert("RGBA")
    pixels = rgba.load()
    w, h = rgba.size

    for y in range(h):
        for x in range(w):
            r, g, b, a = pixels[x, y]
            if a == 0:
                continue
            avg = (r + g + b) / 3
            spread = max(r, g, b) - min(r, g, b)
            if spread <= 14 and avg >= threshold:
                if feather <= 0 or avg >= threshold + feather:
                    pixels[x, y] = (r, g, b, 0)
                else:
                    t = max(0.0, min(1.0, (avg - threshold) / feather))
                    pixels[x, y] = (r, g, b, int((1 - t) * a))

    return rgba


def crop_to_content(img: Image.Image, *, alpha_threshold: int = 8) -> Image.Image:
    rgba = img.convert("RGBA")
    alpha = rgba.split()[3]
    mask = alpha.point(lambda a: 255 if a > alpha_threshold else 0)
    bbox = mask.getbbox()
    if bbox:
        return rgba.crop(bbox)
    return rgba


def fit_to_canvas(
    img: Image.Image,
    *,
    size: int = 1024,
    fill_ratio: float = 0.92,
) -> Image.Image:
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    w, h = img.size
    target = int(size * fill_ratio)
    scale = min(target / w, target / h)
    nw = max(1, int(w * scale))
    nh = max(1, int(h * scale))
    resized = img.resize((nw, nh), Image.Resampling.LANCZOS)
    x = (size - nw) // 2
    y = (size - nh) // 2
    canvas.paste(resized, (x, y), resized)
    return canvas


def has_real_transparency(img: Image.Image, *, min_ratio: float = 0.12) -> bool:
    ratio, _, _ = _alpha_stats(img)
    return ratio >= min_ratio


def process_logo(
    src: Path,
    dst: Path,
    *,
    canvas_size: int = 1024,
    fill_ratio: float = 0.92,
) -> Image.Image:
    img = Image.open(src)
    if has_real_transparency(img):
        transparent = img.convert("RGBA")
    else:
        # Gemini 等「假透明」棋盘格：先泛洪再去浅底
        transparent = flood_remove_light_background(img)
        transparent = remove_light_background(transparent)

    cropped = crop_to_content(transparent)
    fitted = fit_to_canvas(cropped, size=canvas_size, fill_ratio=fill_ratio)
    dst.parent.mkdir(parents=True, exist_ok=True)
    fitted.save(dst, "PNG")
    return fitted


def main() -> int:
    src = Path(
        sys.argv[1]
        if len(sys.argv) > 1
        else r"C:\Users\23282\Desktop\Gemini_Generated_Image_7xhhcl7xhhcl7xhh.png"
    )
    dst = Path(
        sys.argv[2]
        if len(sys.argv) > 2
        else Path(__file__).resolve().parent.parent / "assets" / "logo-transparent.png"
    )
    fill_ratio = float(sys.argv[3]) if len(sys.argv) > 3 else 0.94

    if not src.is_file():
        print(f"源文件不存在: {src}", file=sys.stderr)
        return 1

    out = process_logo(src, dst, fill_ratio=fill_ratio)
    ratio, min_a, max_a = _alpha_stats(out)
    print(
        f"已生成透明 logo: {dst} ({out.size[0]}x{out.size[1]}, "
        f"fill={fill_ratio:.0%}, transparent={ratio:.1%}, alpha={min_a}-{max_a})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
