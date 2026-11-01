<script setup>
import { computed } from "vue"
import { useTasksStore } from "../../../stores/tasks.js"

const props = defineProps({
  task: Object,
})

const tasksStore = useTasksStore()
const pinned = computed(() => tasksStore.pinnedIds.includes(props.task?.taskId))

defineOptions({ name: "TaskPin" })
</script>

<template>
  <v-btn
    icon="mdi-paperclip"
    variant="text"
    density="comfortable"
    size="small"
    :color="pinned ? 'primary' : undefined"
    :class="{ 'task-pin--off': !pinned }"
    :title="pinned ? 'Открепить' : 'Закрепить вверху таблицы'"
    @click.stop="tasksStore.togglePin(task.taskId)"
  />
</template>

<style lang="scss">
.task-pin--off {
  opacity: 0.45;
}
</style>
