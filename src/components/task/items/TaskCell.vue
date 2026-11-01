<script setup>
import TaskActions from "./TaskActions.vue"
import TaskComment from "./TaskComment.vue"
import TaskTime from "./TaskTime.vue"
import TaskStart from "./TaskStart.vue"
import TaskEnd from "./TaskEnd.vue"
import TaskTags from "./TaskTags.vue"
import TaskStatus from "./TaskStatus.vue"
import TaskOurCar from "./TaskOurCar.vue"
import TaskMode from "./TaskMode.vue"
import TaskPin from "./TaskPin.vue"

// Ячейки, которые рисуются отдельными компонентами; остальные колонки — просто текст.
const CELL_COMPONENTS = {
  pin: TaskPin,
  actions: TaskActions,
  comment: TaskComment,
  time: TaskTime,
  start: TaskStart,
  end: TaskEnd,
  tags: TaskTags,
  status: TaskStatus,
  ourCar: TaskOurCar,
  mode: TaskMode,
}

defineProps({
  task: Object,
  columnKey: String,
})

defineEmits(["edit", "delete"])

defineOptions({ name: "TaskCell" })
</script>

<template>
  <component
    :is="CELL_COMPONENTS[columnKey]"
    v-if="CELL_COMPONENTS[columnKey]"
    :task="task"
    @edit="$emit('edit', $event)"
    @delete="$emit('delete', $event)"
  />
  <div v-else-if="columnKey === 'order'" class="order-element">{{ task.order }}</div>
  <template v-else>{{ task[columnKey] }}</template>
</template>
