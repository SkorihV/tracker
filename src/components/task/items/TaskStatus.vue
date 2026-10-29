<script setup>
import { computed, defineProps } from "vue"
import { useDictionariesStore } from "../../../stores/dictionaries.js"
import { useTasksStore } from "../../../stores/tasks.js"
import { statusHex, statusTextHex } from "../../../statusColors.js"
const dicts = useDictionariesStore()
const tasksStore = useTasksStore()
const props = defineProps({
  task: Object,
})

function statusColorOf(row) {
  const st = dicts.statuses.find((s) => s.name === row.status)
  return statusHex(st?.color)
}

const statusCellStyle = (row) => {
  const c = statusColorOf(row)
  return { backgroundColor: c, color: statusTextHex(c) }
}

const statusTextColor = computed(() => statusTextHex(statusColorOf(props.task)))

async function onChangeStatus(row, value) {
  if (value === props.task.status) return
  await tasksStore.updateTaskStatus(row.taskId, value || "")
}
const statusItems = computed(() => dicts.statuses.map((s) => ({ title: s.name, value: s.name })))

defineOptions({ name: "TaskStatus" })
</script>

<template>
  <v-sheet
    height="100%"
    width="100%"
    min-width="150px"
    class="pa-2 status-cell"
    :style="statusCellStyle(task)"
    @click.stop
  >
    <v-autocomplete
      :model-value="task.status"
      :items="statusItems"
      :style="{ color: statusTextColor }"
      density="compact"
      variant="outlined"
      hide-details
      hide-selected
      placeholder="Статус"
      @update:model-value="onChangeStatus(task, $event)"
    />
  </v-sheet>
</template>

<style lang="scss">
.status-cell .v-autocomplete__selection-text {
  border-radius: 6px;
}
.status-cell .v-field__input {
  color: inherit;
}
</style>
