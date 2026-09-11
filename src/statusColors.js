const LEGACY_COLORS = {
  default: "",
  blue: "#1E88E5",
  teal: "#00897B",
  green: "#43A047",
  orange: "#FB8C00",
  red: "#E53935",
  purple: "#8E24AA",
  brown: "#6D4C41",
  "blue-grey": "#546E7A",
  pink: "#D81B60",
};

const HEX_RE = /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;

export function statusHex(color) {
  const c = String(color || "").trim();
  if (HEX_RE.test(c)) return c;
  return LEGACY_COLORS[c] || "";
}