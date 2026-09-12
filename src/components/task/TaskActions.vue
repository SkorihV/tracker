<script setup>
import {defineProps} from 'vue'
import {useAppStore} from "../../store.js";
const store = useAppStore()

const props = defineProps({
  task: Object
})

const emit = defineEmits(["edit", "delete"])


async function onPause(task) {
  if (task.mode === "running") await store.pauseTask(task.taskId);
}

async function onComplete(task) {
  await store.completeTask(task.taskId);
}

async function onStart(task) {
  if (task.mode === "running") return;
  if (task.mode === "paused") await store.resumeTask(task.taskId);
  else await store.startTask(task.taskId);
}
defineOptions({name:'TaskActions'})
</script>



<template>
  <div class="d-flex ga-1">
    <template v-if="task.mode === 'running'">
      <v-btn icon aria-label="Пауза" variant="text" size="x-small" @click.stop="onPause(task)">
        <v-icon icon="systemIcons:iconPause" color="orange" />
      </v-btn>
      <v-btn icon aria-label="Завершить" variant="text" size="x-small" @click.stop="onComplete(task)">
        <v-icon icon="systemIcons:iconStop" color="red"/>
      </v-btn>
    </template>
    <template v-else-if="task.mode === 'paused'">
      <v-btn icon aria-label="Возобновить" variant="text" size="x-small" @click.stop="onStart(task)">
        <v-icon icon="systemIcons:iconPlay" color="green"/>
      </v-btn>
      <v-btn icon aria-label="Завершить" variant="text" size="x-small" @click.stop="onComplete(task)">
        <v-icon icon="systemIcons:iconStop" color="red"/>
      </v-btn>
    </template>
    <template v-else>
      <v-btn icon aria-label="Возобновить" variant="text" size="x-small" @click.stop="onStart(task)">
        <v-icon icon="systemIcons:iconPlay" color="green"/>
      </v-btn>
      <v-btn icon aria-label="Завершить" disabled variant="text" size="x-small" @click.stop="onComplete(task)">
        <v-icon icon="systemIcons:iconStop" color="red"/>
      </v-btn>
    </template>

    <v-btn icon aria-label="Редактировать" variant="text" size="x-small" @click.stop="emit('edit', task)">
      <v-icon icon="systemIcons:iconEdit" />
    </v-btn>
    <v-btn icon aria-label="Удалить" variant="text" size="x-small" color="error" @click.stop="emit('delete', [task.taskId])">
      <v-icon icon="systemIcons:iconTrash" />
    </v-btn>
  </div>
</template>

<style scoped>

</style>