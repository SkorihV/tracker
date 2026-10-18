<script setup>
import { computed } from "vue"

const props = defineProps({
  label: { type: String, default: "" },
  placeholder: { type: String, default: "" },
  clearable: { type: Boolean, default: false },
  clearToToday: { type: Boolean, default: false },
  density: { type: String, default: "compact" },
  variant: { type: String, default: "outlined" },
  hideDetails: { type: Boolean, default: true },
})

const model = defineModel({ type: String, default: "" })

function toParts(v) {
  if (!v) return null
  if (v instanceof Date) {
    if (isNaN(v.getTime())) return null
    return { y: v.getFullYear(), m: v.getMonth() + 1, d: v.getDate() }
  }
  const iso = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(v))
  if (iso) return { y: +iso[1], m: +iso[2], d: +iso[3] }
  const ru = /^(\d{2})\.(\d{2})\.(\d{4})$/.exec(String(v))
  if (ru) return { y: +ru[3], m: +ru[2], d: +ru[1] }
  return null
}

function toIso(v) {
  const p = toParts(v)
  if (!p) return null
  const pad = (n) => String(n).padStart(2, "0")
  return `${p.y}-${pad(p.m)}-${pad(p.d)}`
}

function toRu(v) {
  const p = toParts(v)
  if (!p) return ""
  const pad = (n) => String(n).padStart(2, "0")
  return `${pad(p.d)}.${pad(p.m)}.${p.y}`
}

function dateDisplay(v) {
  return toRu(v)
}

const iso = computed({
  get: () => toIso(model.value),
  set: (v) => (model.value = toRu(v)),
})

function onClear() {
  if (!props.clearToToday) return
  const d = new Date()
  const pad = (n) => String(n).padStart(2, "0")
  model.value = `${pad(d.getDate())}.${pad(d.getMonth() + 1)}.${d.getFullYear()}`
}

defineOptions({
  name: "BaseDateField",
})
</script>

<template>
  <v-date-input
    v-model="iso"
    :label="label"
    :placeholder="placeholder || 'дд.мм.гггг'"
    min-width="180px"
    :clearable="clearable"
    :density="density"
    :variant="variant"
    :hide-details="hideDetails"
    prepend-icon=""
    prepend-inner-icon="mdi-calendar-month-outline"
    input-format="dd.MM.yyyy"
    :display-format="dateDisplay"
    :menu-props="{ location: 'bottom start', maxWidth: 344 }"
    @click:clear="onClear"
  />
</template>