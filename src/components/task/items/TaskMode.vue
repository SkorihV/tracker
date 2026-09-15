<script setup>
import {defineProps} from 'vue'

const props = defineProps({
  task: Object
})

const statusMeta = {
  running: { color: "success", icon: "systemIcons:iconPause" },
  paused: { color: "warning", icon: "systemIcons:iconPlay" },
  completed: { color: "secondary", icon: "systemIcons:iconStop" },
}

const statusLabel = {
  running: "Запущена",
  paused: "На паузе",
  completed: "Завершена",
}

defineOptions({name:'TaskMode'})
</script>

<template>
  <v-chip border :color="statusMeta[task.mode]?.color || 'secondary'" size="large" variant="tonal">
    <template #prepend>
      <v-icon :icon="statusMeta[task.mode]?.icon || 'systemIcons:iconStop'" start />
    </template>
    <span class="text-caption">{{ statusLabel[task.mode] || task.mode }}</span>
  </v-chip>
</template>

<style lang="scss" scoped>

</style>