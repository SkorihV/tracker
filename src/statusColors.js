const HEX_RE = /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;

export function statusHex(color) {
  const c = String(color || "").trim();
  if (HEX_RE.test(c)) return c;
  return "#000000";
}