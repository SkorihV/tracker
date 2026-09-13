const HEX_RE = /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/

export function statusHex(color) {
  const c = String(color || "").trim()
  if (HEX_RE.test(c)) return c
  return "#ffffff"
}

function channel(hex, offset) {
  return parseInt(hex.slice(offset, offset + 2), 16)
}

export function statusTextHex(color) {
  const bg = statusHex(color).slice(0, 7)
  let r, g, b
  if (bg.length === 4) {
    r = parseInt(bg[1] + bg[1], 16)
    g = parseInt(bg[2] + bg[2], 16)
    b = parseInt(bg[3] + bg[3], 16)
  } else {
    r = channel(bg, 1)
    g = channel(bg, 3)
    b = channel(bg, 5)
  }
  const luminance = 0.299 * r + 0.587 * g + 0.114 * b
  return luminance > 140 ? "#000000" : "#ffffff"
}