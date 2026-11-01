import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue"

/** Сколько закреплённых строк видно без прокрутки. */
export const PINNED_VISIBLE_ROWS = 3

/**
 * Блок закреплённых задач под заголовком таблицы: высота ограничена тремя
 * строками, остальные прокручиваются внутри блока. Колонки блока повторяют
 * колонки основной таблицы, а число скрытых сверху и снизу строк считается
 * для кнопок-индикаторов.
 *
 * @param {import("vue").Ref<HTMLElement|null>} wrapRef контейнер таблицы
 * @param {() => unknown} source то, при изменении чего блок надо перемерить
 */
export function usePinnedBlock(wrapRef, source) {
  const hiddenAbove = ref(0)
  const hiddenBelow = ref(0)
  let observer = null
  let hScroller = null

  const query = (selector) => wrapRef.value?.querySelector(selector)

  function measureHidden() {
    const box = query(".pinned-scroll")
    if (!box) {
      hiddenAbove.value = 0
      hiddenBelow.value = 0
      return
    }
    const view = box.getBoundingClientRect()
    let above = 0
    let below = 0
    for (const tr of box.querySelectorAll("tr.pinned-row")) {
      const r = tr.getBoundingClientRect()
      if (r.top < view.top - 1) above++
      else if (r.bottom > view.bottom + 1) below++
    }
    hiddenAbove.value = above
    hiddenBelow.value = below
  }

  /** Кнопки-индикаторы стоят по центру видимой части таблицы, а не всей её ширины. */
  function updateCenter() {
    const view = query(".pinned-view")
    if (view && hScroller) {
      view.style.setProperty("--pin-center", `${hScroller.scrollLeft + hScroller.clientWidth / 2}px`)
    }
  }

  async function update() {
    await nextTick()
    const block = query(".pinned-block")
    if (!block) return measureHidden()

    block.style.setProperty("--pin-top", `${query("thead")?.offsetHeight ?? 0}px`)

    // Ширины колонок — как у основной таблицы.
    const widths = [...wrapRef.value.querySelectorAll("thead tr:not(.v-data-table-progress) th")].map((th) =>
      Math.floor(th.getBoundingClientRect().width)
    )
    const table = block.querySelector("table.pinned-table")
    const cols = [...table.querySelectorAll("col")]
    cols.forEach((col, i) => widths[i] && (col.style.width = `${widths[i]}px`))
    const total = widths.reduce((a, b) => a + b, 0)
    table.style.width = `${total}px`

    // Высота блока — первые три строки; остальные уходят в прокрутку.
    const box = block.querySelector(".pinned-scroll")
    const rows = [...box.querySelectorAll("tr.pinned-row")]
    box.style.maxHeight = `${rows.slice(0, PINNED_VISIBLE_ROWS).reduce((h, tr) => h + tr.offsetHeight, 0)}px`

    // Полоса прокрутки блока забирает ширину у последней колонки, а не выталкивает таблицу.
    const scrollbar = box.offsetWidth - box.clientWidth
    if (scrollbar > 0 && cols.length) {
      table.style.width = `${total - scrollbar}px`
      cols[cols.length - 1].style.width = `${widths[widths.length - 1] - scrollbar}px`
    }

    updateCenter()
    measureHidden()
  }

  function scrollPinned(edge) {
    const box = query(".pinned-scroll")
    box?.scrollTo({ top: edge === "top" ? 0 : box.scrollHeight })
  }

  watch(source, update, { flush: "post" })

  onMounted(() => {
    hScroller = query(".v-table__wrapper")
    hScroller?.addEventListener("scroll", updateCenter, { passive: true })
    if (typeof ResizeObserver !== "undefined") {
      observer = new ResizeObserver(update)
      if (wrapRef.value) observer.observe(wrapRef.value)
      const table = query("table")
      if (table) observer.observe(table)
    }
    update()
  })

  onBeforeUnmount(() => {
    observer?.disconnect()
    hScroller?.removeEventListener("scroll", updateCenter)
  })

  return { hiddenAbove, hiddenBelow, onPinnedScroll: measureHidden, scrollPinned }
}
