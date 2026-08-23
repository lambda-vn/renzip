#!/usr/bin/env python3
"""Turn the drawn logo into an icon source: transparent corners, no margin.

src/assets/logo.png comes out of an image generator as RGB with no alpha
channel, so the rounded corners of the mark were flattened onto black. Used
as is, the application icon shows black triangles at every corner, and the
mark itself sits inside a dead margin that makes it look small at 32 pixels.

This reads that file, gives the background back its transparency, crops the
margin away, and writes src/assets/logo-icon.png, which is what
`npx tauri icon` is pointed at. The original is never modified.

    python scripts/prepare_logo.py
    npx tauri icon src/assets/logo-icon.png

Written against the PNG specification rather than an imaging library: the
project has no Python dependency, and adding one for a file that changes
once a year is a poor trade.
"""

import struct
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "src" / "assets" / "logo.png"
TARGET = ROOT / "src" / "assets" / "logo-icon.png"

# A pixel counts as background while it stays below this share of the mark's
# own brightness. The rounded edge is anti-aliased, so the band between the
# black corner and the solid colour has to be walked, not cut.
#
# Well below 1 on purpose: the mark is filled with a gradient, so its own
# brightness wanders a few percent around the reference. A threshold hugging
# 1 would let the walk step inside the mark wherever the gradient dips, and
# leave the square very slightly see through.
EDGE = 0.85


def read_png(path):
    """Decode a non-interlaced 8 bit PNG.

    Args:
        path: File to read.

    Returns:
        A (width, height, channels, pixels) tuple, pixels being a bytearray
        of width * height * channels bytes.

    Raises:
        SystemExit: If the file is not a PNG this script can read.
    """
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"{path} is not a PNG")

    header = None
    payload = bytearray()
    offset = 8
    while offset < len(data):
        length = struct.unpack(">I", data[offset : offset + 4])[0]
        kind = data[offset + 4 : offset + 8]
        body = data[offset + 8 : offset + 8 + length]
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", body)
        elif kind == b"IDAT":
            payload += body
        elif kind == b"IEND":
            break
        offset += 12 + length

    if header is None:
        raise SystemExit(f"{path} has no header")
    width, height, depth, colour, _, _, interlace = header
    if depth != 8 or interlace != 0 or colour not in (2, 6):
        raise SystemExit(
            f"{path}: only 8 bit RGB or RGBA without interlacing is handled"
        )

    channels = 3 if colour == 2 else 4
    raw = zlib.decompress(bytes(payload))
    return width, height, channels, unfilter(raw, width, height, channels)


def unfilter(raw, width, height, channels):
    """Undo the per scanline filters a PNG stores its rows with.

    Args:
        raw: Inflated image data, one filter byte per scanline.
        width: Image width in pixels.
        height: Image height in pixels.
        channels: Bytes per pixel.

    Returns:
        The pixels, without filter bytes.
    """
    stride = width * channels
    out = bytearray(height * stride)
    previous = bytearray(stride)

    for y in range(height):
        start = y * (stride + 1)
        kind = raw[start]
        line = bytearray(raw[start + 1 : start + 1 + stride])

        if kind == 1:
            for i in range(channels, stride):
                line[i] = (line[i] + line[i - channels]) & 0xFF
        elif kind == 2:
            for i in range(stride):
                line[i] = (line[i] + previous[i]) & 0xFF
        elif kind == 3:
            for i in range(stride):
                left = line[i - channels] if i >= channels else 0
                line[i] = (line[i] + ((left + previous[i]) >> 1)) & 0xFF
        elif kind == 4:
            for i in range(stride):
                left = line[i - channels] if i >= channels else 0
                up = previous[i]
                upleft = previous[i - channels] if i >= channels else 0
                estimate = left + up - upleft
                da, db, dc = (
                    abs(estimate - left),
                    abs(estimate - up),
                    abs(estimate - upleft),
                )
                if da <= db and da <= dc:
                    nearest = left
                elif db <= dc:
                    nearest = up
                else:
                    nearest = upleft
                line[i] = (line[i] + nearest) & 0xFF
        elif kind != 0:
            raise SystemExit(f"unknown scanline filter {kind}")

        out[y * stride : (y + 1) * stride] = line
        previous = line
    return out


def to_rgba(width, height, channels, pixels):
    """Give the flattened background its transparency back.

    The mark was drawn over black, so a corner pixel holds alpha * colour and
    nothing else. Walking outward from the frame, every pixel dimmer than the
    mark itself is background: its brightness is exactly how opaque it was.

    Args:
        width: Image width.
        height: Image height.
        channels: Bytes per pixel of `pixels`.
        pixels: Image data.

    Returns:
        A bytearray of width * height * 4 bytes.
    """
    if channels == 4:
        return bytearray(pixels)

    def rgb(index):
        base = index * 3
        return pixels[base], pixels[base + 1], pixels[base + 2]

    # Brightness of the mark where it meets the background, which is what the
    # walk below has to stop at. Taken from the edge itself rather than from
    # the brightest pixel of the image: the white drawing inside the mark is
    # brighter than the coloured square around it, and using that as the
    # reference would let the walk eat the square as well.
    peak = 1
    middle = width // 2
    for y in range(height):
        if max(rgb(y * width + middle)) <= 60:
            continue
        # Entering the mark lands on its anti-aliased edge, which is darker
        # than the mark itself; a few rows further in, the colour is solid.
        peak = max(max(rgb((y + step) * width + middle)) for step in range(8))
        break

    out = bytearray(width * height * 4)
    for index in range(width * height):
        red, green, blue = rgb(index)
        out[index * 4 : index * 4 + 4] = bytes((red, green, blue, 255))

    # Flood fill from the frame, so a dark pixel inside the mark is left alone.
    seen = bytearray(width * height)
    stack = []
    for x in range(width):
        stack.append(x)
        stack.append((height - 1) * width + x)
    for y in range(height):
        stack.append(y * width)
        stack.append(y * width + width - 1)

    while stack:
        index = stack.pop()
        if seen[index]:
            continue
        seen[index] = 1
        red, green, blue = rgb(index)
        share = max(red, green, blue) / peak
        if share >= EDGE:
            continue
        # Below a few percent the pixel is invisible, and dividing such a
        # value by its own opacity below would only amplify rounding noise.
        alpha = round(255 * share) if share > 0.03 else 0
        out[index * 4 + 3] = alpha
        if alpha > 0:
            # The stored colour is the edge colour already multiplied by its
            # opacity against black. Left as is, a half transparent pixel
            # keeps that darkening and draws a grey halo on a light desktop,
            # so the multiplication is undone.
            scale = 255 / alpha
            out[index * 4] = min(255, round(red * scale))
            out[index * 4 + 1] = min(255, round(green * scale))
            out[index * 4 + 2] = min(255, round(blue * scale))

        x, y = index % width, index // width
        if x > 0:
            stack.append(index - 1)
        if x + 1 < width:
            stack.append(index + 1)
        if y > 0:
            stack.append(index - width)
        if y + 1 < height:
            stack.append(index + width)
    return out


def crop(width, height, rgba):
    """Cut the fully transparent margin away.

    Args:
        width: Image width.
        height: Image height.
        rgba: Image data, four bytes per pixel.

    Returns:
        A (width, height, pixels) tuple of the cropped image.
    """
    left, top, right, bottom = width, height, -1, -1
    for y in range(height):
        row = y * width
        for x in range(width):
            if rgba[(row + x) * 4 + 3] > 8:
                left = min(left, x)
                right = max(right, x)
                top = min(top, y)
                bottom = max(bottom, y)

    if right < left or bottom < top:
        return width, height, rgba

    new_width, new_height = right - left + 1, bottom - top + 1
    out = bytearray(new_width * new_height * 4)
    for y in range(new_height):
        start = ((top + y) * width + left) * 4
        out[y * new_width * 4 : (y + 1) * new_width * 4] = rgba[
            start : start + new_width * 4
        ]
    return new_width, new_height, out


def square(width, height, rgba):
    """Centre the image on a square, transparent canvas.

    `tauri icon` refuses anything but a square source, and trimming the
    margin almost never lands on one: the drawing is a few pixels taller than
    it is wide. Padding rather than stretching keeps the circles round.

    Args:
        width: Image width.
        height: Image height.
        rgba: Image data, four bytes per pixel.

    Returns:
        A (side, pixels) tuple.
    """
    side = max(width, height)
    if side == width == height:
        return side, rgba

    left, top = (side - width) // 2, (side - height) // 2
    out = bytearray(side * side * 4)
    for y in range(height):
        start = ((top + y) * side + left) * 4
        out[start : start + width * 4] = rgba[y * width * 4 : (y + 1) * width * 4]
    return side, out


def write_png(path, width, height, rgba):
    """Write an 8 bit RGBA PNG.

    Args:
        path: File to write.
        width: Image width.
        height: Image height.
        rgba: Image data, four bytes per pixel.
    """
    stride = width * 4
    raw = bytearray()
    for y in range(height):
        raw.append(0)
        raw += rgba[y * stride : (y + 1) * stride]

    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def chunk(kind, payload):
    """Frame one PNG chunk, length and CRC included.

    Args:
        kind: Four byte chunk type.
        payload: Chunk content.

    Returns:
        The complete chunk.
    """
    body = kind + payload
    return struct.pack(">I", len(payload)) + body + struct.pack(">I", zlib.crc32(body))


def main():
    """Read the drawn logo and write the icon source next to it."""
    width, height, channels, pixels = read_png(SOURCE)
    rgba = to_rgba(width, height, channels, pixels)
    cropped_width, cropped_height, cropped = crop(width, height, rgba)
    side, padded = square(cropped_width, cropped_height, cropped)
    write_png(TARGET, side, side, padded)
    print(
        f"{SOURCE.name} {width}x{height} -> {TARGET.name} {side}x{side} "
        f"(mark {cropped_width}x{cropped_height}), {TARGET.stat().st_size // 1024} KiB"
    )


if __name__ == "__main__":
    main()
