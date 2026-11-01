/**
 * Описание колонок таблицы задач и слияние с порядком/видимостью из настроек.
 * Ширина колонок с «тяжёлыми» ячейками (статус, действия, режим, наша машина) задана явно:
 * иначе при поиске без совпадений или без таких значений таблица пересчитывает их по
 * содержимому и колонки схлопываются.
 */

export function defaultColumns() {
  return [
    { key: "time", label: "Время", visible: true, width: "100px" },
    { key: "status", label: "Статус", visible: true, width: "182px", minWidth: 182 },
    { key: "ourCar", label: "Наша машина", visible: true, width: "92px", minWidth: 92 },
    { key: "actions", label: "Действия", visible: true, width: "142px", minWidth: 142 },
    { key: "mode", label: "Режим", visible: true, width: "155px", minWidth: 155 },
    { key: "order", label: "Заявка", visible: true, width: "500px" },
    { key: "client", label: "Клиент", visible: true, width: "" },
    { key: "tags", label: "Тег", visible: true, width: "" },
    { key: "start", label: "Начало", visible: true, width: "200px" },
    { key: "end", label: "Завершение", visible: true, width: "200px" },
    { key: "user", label: "Пользователь", visible: true, width: "" },
    { key: "comment", label: "Комментарий", visible: true, width: "", maxWidth: 500 },
    { key: "taskId", label: "ID", visible: false, width: "" },
  ]
}

/**
 * Применить сохранённые [{ key, visible }] к описаниям колонок: порядок берётся
 * из prefs, неизвестные ключи отбрасываются, недостающие колонки добавляются в конец.
 */
export function mergeColumns(prefs) {
  const defs = defaultColumns()
  if (!Array.isArray(prefs) || prefs.length === 0) return defs
  const byKey = new Map(defs.map((d) => [d.key, d]))
  const out = []
  const used = new Set()
  for (const p of prefs) {
    const d = typeof p.key === "string" ? byKey.get(p.key) : null
    if (d && !used.has(p.key)) {
      used.add(p.key)
      out.push({ ...d, visible: p.visible !== false })
    }
  }
  for (const d of defs) if (!used.has(d.key)) out.push(d)
  return out
}

/** Порядок колонок по умолчанию; видимость текущих колонок сохраняется. */
export function resetColumnOrder(current) {
  const byKey = new Map(current.map((c) => [c.key, c]))
  const out = defaultColumns().map((d) => byKey.get(d.key) || d)
  const used = new Set(out.map((c) => c.key))
  for (const c of current) if (!used.has(c.key)) out.push(c)
  return out
}
