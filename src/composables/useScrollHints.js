import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue"

/**
 * Подсказки прокрутки основной таблицы: сколько строк скрыто выше и ниже
 * видимой области (между закреплённым блоком и нижним краем). Закреплённый блок
 * и заголовок в видимую область не входят.
 *
 * @param {import("vue").Ref<HTMLElement|null>} wrapRef контейнер таблицы
 * @param {() => unknown} source то, при изменении чего подсчёт надо повторить
 */
export function useScrollHints(wrapRef, source) {
  const above = ref(0)
  const below = ref(0)
  // Отступы кнопок от краёв контейнера: под заголовком/закреплённым блоком и над полосой прокрутки.
  const topOffset = ref(0)
  const bottomOffset = ref(0)
  // Центр видимой области таблицы (без вертикальной полосы прокрутки) — как у кнопок закреплённого блока.
  const centerX = ref(0)
  let scroller = null
  let observer = null
  let frame = 0

  function measure() {
    frame = 0
    const wrap = wrapRef.value
    scroller = wrap?.querySelector(".v-table__wrapper") ?? null
    if (!wrap || !scroller) return
    const box = scroller.getBoundingClientRect()
    const header = wrap.querySelector("thead")?.offsetHeight ?? 0
    const pinned = wrap.querySelector(".pinned-block")?.offsetHeight ?? 0
    const viewTop = box.top + header + pinned
    const viewBottom = box.top + scroller.clientHeight

    let hiddenAbove = 0
    let hiddenBelow = 0
    for (const tr of scroller.querySelectorAll(
      ":scope > table > tbody > tr:not(.pinned-block):not(.v-data-table-group-header-row)"
    )) {
      const r = tr.getBoundingClientRect()
      if (r.top < viewTop - 1) hiddenAbove++
      else if (r.bottom > viewBottom + 1) hiddenBelow++
    }
    above.value = hiddenAbove
    below.value = hiddenBelow
    topOffset.value = header + pinned + 6
    bottomOffset.value = scroller.offsetHeight - scroller.clientHeight + 6
    centerX.value = box.left - wrap.getBoundingClientRect().left + scroller.clientWidth / 2
  }

  /** Подсчёт не чаще раза за кадр: прокрутка и таймер обновления строк могут идти часто. */
  function schedule() {
    if (!frame) frame = requestAnimationFrame(measure)
  }

  function scrollMain(edge) {
    scroller?.scrollTo({ top: edge === "top" ? 0 : scroller.scrollHeight })
  }

  watch(source, () => nextTick(schedule), { flush: "post" })

  onMounted(() => {
    scroller = wrapRef.value?.querySelector(".v-table__wrapper") ?? null
    scroller?.addEventListener("scroll", schedule, { passive: true })
    if (typeof ResizeObserver !== "undefined") {
      observer = new ResizeObserver(schedule)
      if (wrapRef.value) observer.observe(wrapRef.value)
      const table = wrapRef.value?.querySelector("table")
      if (table) observer.observe(table)
    }
    schedule()
  })

  onBeforeUnmount(() => {
    observer?.disconnect()
    scroller?.removeEventListener("scroll", schedule)
    if (frame) cancelAnimationFrame(frame)
  })

  return { above, below, topOffset, bottomOffset, centerX, scrollMain }
}
