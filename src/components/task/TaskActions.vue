<script setup>
import {defineProps} from 'vue'
import {useAppStore} from "../../store.js"
const store = useAppStore()

const props = defineProps({
  task: Object
})

const emit = defineEmits(["edit", "delete"])


async function onPause(task) {
  if (task.mode === "running") await store.pauseTask(task.taskId)
}

async function onComplete(task) {
  await store.completeTask(task.taskId)
}

async function onStart(task) {
  if (task.mode === "running") return
  if (task.mode === "paused") await store.resumeTask(task.taskId)
  else await store.startTask(task.taskId)
}
defineOptions({name:'TaskActions'})
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
      <v-sheet disabled v-tooltip:bottom="'Завершить'" color="transparent" @click.stop="onComplete(task)">
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

<style scoped>

</style>