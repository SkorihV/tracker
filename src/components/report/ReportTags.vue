<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue"
import { useDictionariesStore } from "../../stores/dictionaries.js"

const props = defineProps({
  /** Итоги по тегам: [{ name, timeLabel, actualLabel }] */
  tags: { type: Array, default: () => [] },
})

const NO_TAG = "(без тега)"
const GAP = 8
const MORE_BUTTON_WIDTH = 84

const dicts = useDictionariesStore()

/** «Без тега» первым, дальше — в порядке справочника; теги не из справочника — в конце по алфавиту. */
const ordered = computed(() => {
  const position = new Map(dicts.tags.map((t, i) => [t.name, i]))
  const rank = (name) => (name === NO_TAG ? -1 : (position.get(name) ?? Number.MAX_SAFE_INTEGER))
  return [...props.tags].sort((a, b) => rank(a.name) - rank(b.name) || a.name.localeCompare(b.name, "ru"))
})

const label = (t) => `${t.name === NO_TAG ? "Без тега" : t.name}: ${t.timeLabel} (факт. ${t.actualLabel})`

const expanded = ref(false)
const visibleCount = ref(0)
const rowRef = ref(null)
const titleRef = ref(null)
const ghostRef = ref(null)
let observer = null

/** Сколько чипов помещается в одну строку вместе с кнопкой «еще N». */
function recalc() {
  const row = rowRef.value
  const ghost = ghostRef.value
  if (!row || !ghost) return
  const widths = [...ghost.children].map((chip) => chip.offsetWidth)
  const available = row.clientWidth - (titleRef.value?.offsetWidth ?? 0) - GAP
  let used = 0
  let fit = 0
  for (let i = 0; i < widths.length; i++) {
    const next = used + widths[i] + (i ? GAP : 0)
    const needsButton = i < widths.length - 1
    if (next + (needsButton ? GAP + MORE_BUTTON_WIDTH : 0) > available) break
    used = next
    fit++
  }
  visibleCount.value = Math.max(fit, 1)
}

const shown = computed(() => (expanded.value ? ordered.value : ordered.value.slice(0, visibleCount.value)))
const hiddenCount = computed(() => ordered.value.length - Math.min(visibleCount.value, ordered.value.length))

watch(ordered, () => nextTick(recalc))

onMounted(() => {
  recalc()
  observer = new ResizeObserver(recalc)
  observer.observe(rowRef.value)
})
onBeforeUnmount(() => observer?.disconnect())

defineOptions({ name: "ReportTags" })
</script>

<template>
  <div v-if="ordered.length" ref="rowRef" class="report-tags">
    <div class="report-tags__line" :class="{ 'report-tags__line--expanded': expanded }">
      <span ref="titleRef" class="text-caption text-medium-emphasis report-tags__title">По тегам:</span>
      <v-chip v-for="t in shown" :key="t.name" size="small" variant="tonal">{{ label(t) }}</v-chip>
      <v-btn
        v-if="expanded"
        size="x-small"
        variant="text"
        color="primary"
        append-icon="mdi-chevron-up"
        @click="expanded = false"
      >
        Свернуть
      </v-btn>
      <v-btn
        v-else-if="hiddenCount"
        size="x-small"
        variant="tonal"
        color="primary"
        append-icon="mdi-chevron-down"
        @click="expanded = true"
      >
        еще {{ hiddenCount }}
      </v-btn>
    </div>
    <!-- Невидимая копия всех чипов: по ней считается, сколько помещается в одну строку. -->
    <div ref="ghostRef" class="report-tags__ghost" aria-hidden="true">
      <v-chip v-for="t in ordered" :key="t.name" size="small" variant="tonal">{{ label(t) }}</v-chip>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.report-tags {
  position: relative;
  overflow: hidden;
}

.report-tags__line {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  gap: 8px;
  overflow: hidden;
}

// Раскрытый список занимает не больше трёх строк и прокручивается сам.
.report-tags__line--expanded {
  flex-wrap: wrap;
  max-height: 96px;
  overflow-y: auto;
}

.report-tags__title {
  flex: 0 0 auto;
}

.report-tags__ghost {
  position: absolute;
  top: 0;
  left: 0;
  display: flex;
  gap: 8px;
  white-space: nowrap;
  visibility: hidden;
  pointer-events: none;
}
</style>
