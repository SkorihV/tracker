import { onBeforeUnmount, reactive } from "vue"
import { readColumnWidths, saveColumnWidths } from "../utils/cookies"

const MIN_COLUMN_WIDTH = 50
const RESIZE_ZONE = 25

function parsePx(v) {
  if (typeof v === "number" && Number.isFinite(v)) return v
  if (typeof v === "string") {
    const n = parseFloat(v)
    if (Number.isFinite(n)) return n
  }
  return null
}

/**
 * Ресайз колонок таблицы мышью: зона у правого края th, минимум 50px,
 * rAF на mousemove, двойной клик по зоне сбрасывает ширину. Ширины
 * запоминаются в cookie.
 *
 * @param {object} opts
 * @param {() => Array<{ key: string }>} opts.getHeaders заголовки в порядке th
 * @param {() => Array<object>} opts.getColumns колонки с minWidth/maxWidth
 * @param {string[]} opts.resizableKeys колонки, ширину которых можно менять
 */
export function useColumnResize({ getHeaders, getColumns, resizableKeys }) {
  const resizable = new Set(resizableKeys)
  const widths = reactive(readColumnWidths())
  let resizing = null
  let zoneTh = null
  let rafId = null
  let justResized = false

  /** Ограничить ширину declared minWidth/maxWidth колонки (и глобальным минимумом). */
  function clampWidth(col, w) {
    const min = parsePx(col?.minWidth) ?? MIN_COLUMN_WIDTH
    const max = parsePx(col?.maxWidth) ?? Infinity
    return Math.min(max, Math.max(min, w))
  }

  function headerOf(th) {
    const tr = th.parentElement
    return tr ? getHeaders()[[...tr.children].indexOf(th)] : undefined
  }

  const isResizableTh = (th) => resizable.has(headerOf(th)?.key)
  const inResizeZone = (th, e) => th.getBoundingClientRect().right - e.clientX <= RESIZE_ZONE

  function onMousemove(e) {
    if (resizing) return
    const th = e.target.closest?.("th")
    const ok = th && isResizableTh(th) && inResizeZone(th, e)
    if (zoneTh && (!ok || zoneTh !== th)) {
      zoneTh.classList.remove("col-resize-zone")
      zoneTh = null
    }
    if (ok) {
      th.classList.add("col-resize-zone")
      zoneTh = th
    }
  }

  function onMousedown(e) {
    if (e.button !== 0) return
    const th = e.target.closest?.("th")
    if (!th || !isResizableTh(th) || !inResizeZone(th, e)) return
    e.preventDefault()
    e.stopPropagation()
    resizing = { th, key: headerOf(th).key, startX: e.clientX, startWidth: th.offsetWidth }
    th.classList.add("col-resize-active")
    document.body.classList.add("tt-col-resizing")
    document.addEventListener("mousemove", onDocMousemove)
    document.addEventListener("mouseup", onDocMouseup)
  }

  function onDocMousemove(e) {
    if (!resizing) return
    if (rafId) cancelAnimationFrame(rafId)
    rafId = requestAnimationFrame(() => {
      rafId = null
      if (!resizing) return
      const col = getColumns().find((c) => c.key === resizing.key)
      widths[resizing.key] = clampWidth(col, resizing.startWidth + (e.clientX - resizing.startX))
    })
  }

  function onDocMouseup() {
    if (rafId) cancelAnimationFrame(rafId)
    rafId = null
    document.removeEventListener("mousemove", onDocMousemove)
    document.removeEventListener("mouseup", onDocMouseup)
    if (!resizing) return
    resizing.th.classList.remove("col-resize-active")
    document.body.classList.remove("tt-col-resizing")
    resizing = null
    zoneTh = null
    justResized = true
    saveColumnWidths({ ...widths })
  }

  /** Двойной клик по зоне ресайза — сброс ширины колонки к умолчанию. */
  function onDblclick(e) {
    const th = e.target.closest?.("th")
    if (!th || !isResizableTh(th) || !inResizeZone(th, e)) return
    delete widths[headerOf(th).key]
    saveColumnWidths({ ...widths })
  }

  /** Клик после ресайза не должен переключать сортировку. */
  function onClickCapture(e) {
    if (justResized) {
      e.stopPropagation()
      e.preventDefault()
      justResized = false
    }
  }

  onBeforeUnmount(onDocMouseup)

  return { widths, clampWidth, onMousemove, onMousedown, onDblclick, onClickCapture }
}
