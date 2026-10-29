<script setup>
import { defineProps } from "vue"
import { useTasksStore } from "../../../stores/tasks.js"
const tasksStore = useTasksStore()

defineProps({
  task: Object,
})

const emit = defineEmits(["edit", "delete"])

async function onPause(task) {
  if (task.mode === "running") await tasksStore.taskAction(task.taskId, "pause")
}

async function onComplete(task) {
  await tasksStore.taskAction(task.taskId, "complete")
}

async function onStart(task) {
  if (task.mode === "running") return
  if (task.mode === "paused") await tasksStore.taskAction(task.taskId, "resume")
  else await tasksStore.taskAction(task.taskId, "start")
}
defineOptions({ name: "TaskActions" })
</script>

<template>
  <div class="d-flex ga-1">
    <template v-if="task.mode === 'running'">
      <v-sheet v-tooltip:bottom="'Пауза'" color="transparent" @click.stop="onPause(task)">
        <v-icon size="large" icon="systemIcons:iconPause" color="orange" />
      </v-sheet>
      <v-sheet v-tooltip:bottom="'Завершить'" color="transparent" @click.stop="onComplete(task)">
        <v-icon size="large" icon="systemIcons:iconStop" color="red" />
      </v-sheet>
    </template>
    <template v-else-if="task.mode === 'paused'">
      <v-sheet v-tooltip:bottom="'Возобновить'" color="transparent" @click.stop="onStart(task)">
        <v-icon size="large" icon="systemIcons:iconPlay" color="green" />
      </v-sheet>
      <v-sheet v-tooltip:bottom="'Завершить'" disabled color="transparent" @click.stop="onComplete(task)">
        <v-icon size="large" icon="systemIcons:iconStop" color="red" />
      </v-sheet>
    </template>
    <template v-else>
      <v-sheet v-tooltip:bottom="'Возобновить'" color="transparent" @click.stop="onStart(task)">
        <v-icon size="large" icon="systemIcons:iconPlay" color="green" />
      </v-sheet>
      <v-sheet disabled color="transparent">
        <v-icon size="large" icon="systemIcons:iconStop" color="grey" />
      </v-sheet>
    </template>

    <v-sheet v-tooltip:bottom="'Редактировать'" color="transparent" @click.stop="emit('edit', task)">
      <v-icon size="large" icon="systemIcons:iconEdit" color="purple" />
    </v-sheet>
    <v-sheet v-tooltip:bottom="'Удалить'" color="transparent" @click.stop="emit('delete', [task.taskId])">
      <v-icon size="large" icon="systemIcons:iconTrash" color="error" />
    </v-sheet>
  </div>
</template>

<style scoped></style>
