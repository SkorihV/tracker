<script setup>
import {defineProps} from 'vue'
import {fmtTd} from "../../../api.js"

const props = defineProps({
  item: Object,
  columns: Array,
  toggleGroup: Function,
  isGroupOpen: Function,
})

function groupLabel(item) {
  if (item.key === "_day") {
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(item.value || "")
    return m ? `${m[3]}.${m[2]}.${m[1]}` : item.value || "—"
  }
  return item.value || "(не указан)"
}

function groupTime(item) {
  const tasks = (item.items || []).map((x) => (x && x.raw ? x.raw : x))
  const total = tasks.reduce((s, t) => s + (t.seconds || 0), 0)
  return { count: tasks.length, time: fmtTd(total) }
}

defineOptions({name:'TaskGroupHeader'})
</script>

<template>
  <tr class="task-group-row">
    <td :colspan="columns?.length || 0">
      <div class="d-flex align-center ga-1">
        <v-btn icon size="x-small" variant="text" density="default" @click="toggleGroup && toggleGroup(item)">
          <v-icon size="x-large">{{ isGroupOpen && isGroupOpen(item) ? 'mdi-chevron-down' : 'mdi-chevron-right' }}</v-icon>
        </v-btn>
        <span class="font-weight-bold text-body-2">{{ groupLabel(item) }}</span>
        <span class="text-medium-emphasis text-caption ml-1">
          {{ groupTime(item).count }} задач · {{ groupTime(item).time }}
        </span>
      </div>
    </td>
  </tr>
</template>

<style lang="scss">
.task-table .task-group-row td {
  background: rgb(var(--v-theme-surface-variant));
  font-weight: 600;
}
</style>