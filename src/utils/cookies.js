/** Простые настройки интерфейса, которые хранятся в cookie, а не в файле данных. */

export function readCookie(name) {
  try {
    const m = document.cookie
      .split(";")
      .map((s) => s.trim())
      .find((s) => s.startsWith(`${name}=`))
    return m ? decodeURIComponent(m.slice(name.length + 1)) : ""
  } catch {
    return ""
  }
}

export function writeCookie(name, value, maxAge = 31536000) {
  try {
    document.cookie = `${name}=${encodeURIComponent(value)};path=/;max-age=${maxAge}`
  } catch {
    /* cookie недоступны — настройка просто не запоминается */
  }
}

const MIN_COLUMN_WIDTH = 50
const MAX_COLUMN_WIDTH = 5000

/** Ширины колонок таблицы: { [key]: px } */
export function readColumnWidths() {
  try {
    const obj = JSON.parse(readCookie("tt_colwidth") || "{}")
    if (!obj || typeof obj !== "object" || Array.isArray(obj)) return {}
    const out = {}
    for (const [k, w] of Object.entries(obj)) {
      const n = Math.round(Number(w))
      if (Number.isFinite(n) && n >= MIN_COLUMN_WIDTH && n <= MAX_COLUMN_WIDTH) out[k] = n
    }
    return out
  } catch {
    return {}
  }
}

export function saveColumnWidths(widths) {
  writeCookie("tt_colwidth", JSON.stringify(widths || {}))
}

/** Сколько задач можно закрепить: список хранится в cookie (лимит ~4 КБ). */
export const MAX_PINNED = 100

/** Закреплённые задачи: массив taskId в порядке закрепления. */
export function readPinnedIds() {
  try {
    const arr = JSON.parse(readCookie("tt_pinned") || "[]")
    return Array.isArray(arr) ? arr.filter((id) => typeof id === "string").slice(0, MAX_PINNED) : []
  } catch {
    return []
  }
}

export function savePinnedIds(ids) {
  writeCookie("tt_pinned", JSON.stringify(ids.slice(0, MAX_PINNED)))
}
