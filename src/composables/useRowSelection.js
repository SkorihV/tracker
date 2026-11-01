const INTERACTIVE = "input, textarea, select, .v-btn, .v-select, .v-autocomplete, .v-menu, .v-list-item"

/**
 * Выбор строк таблицы кликом: обычный клик переключает строку,
 * Shift+клик выделяет диапазон от первой выбранной строки.
 *
 * @param {import("vue").Ref<Array<{ taskId: string }>>} rows видимые строки
 * @param {import("vue").Ref<Set<string>>} selectedIds выбранные taskId (v-model)
 */
export function useRowSelection(rows, selectedIds) {
  function rowProps({ internalItem }) {
    const id = internalItem?.raw?.taskId
    return selectedIds.value.has(id) ? { class: "row-selected" } : {}
  }

  function onRowClick(event, { item }) {
    const id = item?.taskId
    if (id == null) return
    if (event.target instanceof Element && event.target.closest(INTERACTIVE)) return

    const next = new Set(selectedIds.value || new Set())

    if (event.shiftKey && rows.value.length) {
      const visible = rows.value.filter((r) => r.taskId != null).map((r) => r.taskId)
      const anchor = visible.find((x) => next.has(x)) ?? id
      const from = visible.indexOf(anchor)
      const to = visible.indexOf(id)
      if (from >= 0 && to >= 0) {
        const [lo, hi] = [Math.min(from, to), Math.max(from, to)]
        for (let i = lo; i <= hi; i++) next.add(visible[i])
        selectedIds.value = next
      }
      return
    }

    if (next.has(id)) next.delete(id)
    else next.add(id)
    selectedIds.value = next
  }

  return { rowProps, onRowClick }
}
