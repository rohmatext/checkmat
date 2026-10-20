"""Generate icons/*.png + icon.ico from src-tauri/icons.png, stdlib only (no PIL).
Box-average downscale + multi-size ICO.
"""
import os
import struct
import zlib


def read_png(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    pos, idat = 8, b""
    while pos < len(data):
        (length,) = struct.unpack(">I", data[pos : pos + 4])
        ctype = data[pos + 4 : pos + 8]
        chunk = data[pos + 8 : pos + 8 + length]
        if ctype == b"IHDR":
            w, h, bd, ct, comp, filt, inter = struct.unpack(">IIBBBBB", chunk)
            assert bd == 8 and inter == 0 and ct in (2, 6), "need 8-bit RGB/RGBA non-interlaced"
        elif ctype == b"IDAT":
            idat += chunk
        elif ctype == b"IEND":
            break
        pos += 12 + length

    ch = 4 if ct == 6 else 3
    stride = w * ch
    raw = zlib.decompress(idat)
    out, prev, p = bytearray(), bytearray(stride), 0
    for _ in range(h):
        f = raw[p]
        line = bytearray(raw[p + 1 : p + 1 + stride])
        p += 1 + stride
        for i in range(stride):
            a = line[i - ch] if i >= ch else 0
            b = prev[i]
            c = prev[i - ch] if i >= ch else 0
            if f == 1:
                line[i] = (line[i] + a) & 0xFF
            elif f == 2:
                line[i] = (line[i] + b) & 0xFF
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 0xFF
            elif f == 4:
                q = a + b - c
                pa, pb, pc = abs(q - a), abs(q - b), abs(q - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        out += line
        prev = line
    if ch == 3:
        px = [(out[i], out[i + 1], out[i + 2], 255) for i in range(0, len(out), 3)]
    else:
        px = [tuple(out[i : i + 4]) for i in range(0, len(out), 4)]
    return w, h, px


def resize(w, h, px, tw, th):
    out = bytearray(tw * th * 4)
    for y in range(th):
        y0, y1 = y * h // th, max(y * h // th + 1, (y + 1) * h // th)
        for x in range(tw):
            x0, x1 = x * w // tw, max(x * w // tw + 1, (x + 1) * w // tw)
            r = g = b = a = n = 0
            for sy in range(y0, y1):
                row = sy * w
                for sx in range(x0, x1):
                    pr, pg, pb, pa = px[row + sx]
                    r += pr
                    g += pg
                    b += pb
                    a += pa
                    n += 1
            i = (y * tw + x) * 4
            out[i : i + 4] = bytes((r // n, g // n, b // n, a // n))
    return bytes(out)


def png(size, rgba):
    def chunk(tag, data):
        body = tag + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    raw = b"".join(b"\x00" + rgba[y * size * 4 : (y + 1) * size * 4] for y in range(size))
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def ico(sizes, pngs):
    header = struct.pack("<HHH", 0, 1, len(sizes))
    entries, blob, off = b"", b"", 6 + 16 * len(sizes)
    for s, p in zip(sizes, pngs):
        entries += struct.pack("<BBBBHHII", s % 256, s % 256, 0, 0, 1, 32, len(p), off)
        blob += p
        off += len(p)
    return header + entries + blob


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    src = os.path.join(here, "src-tauri", "icons.png")
    out = os.path.join(here, "src-tauri", "icons")
    os.makedirs(out, exist_ok=True)

    w, h, px = read_png(src)
    sizes = [16, 32, 48, 64, 128, 256]
    pngs = []
    for s in sizes:
        print(f"  resize {w}x{h} -> {s}x{s}")
        pngs.append(png(s, resize(w, h, px, s, s)))

    for name, size in (("32x32.png", 32), ("128x128.png", 128), ("128x128@2x.png", 256)):
        with open(os.path.join(out, name), "wb") as f:
            f.write(pngs[sizes.index(size)])
        print(f"  tulis {name}")

    with open(os.path.join(out, "icon.ico"), "wb") as f:
        f.write(ico(sizes, pngs))
    print("ikon dibuat di", out)


if __name__ == "__main__":
    main()
