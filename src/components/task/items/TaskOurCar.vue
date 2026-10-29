<script setup>
import { useSettingsStore } from "../../../stores/settings.js"
import { statusHex, statusTextHex } from "../../../statusColors.js"
const props = defineProps({
  task: Object,
})

const settingsStore = useSettingsStore()

defineOptions({ name: "TaskOurCar" })
</script>

<template>
  <v-sheet
    v-if="props.task?.ourCar"
    height="100%"
    width="100%"
    min-width="60px"
    class="pa-2 our-car-cell d-flex align-center justify-center"
    :style="{
      backgroundColor: statusHex(settingsStore.settings.ourCarColor),
      color: statusTextHex(settingsStore.settings.ourCarColor),
    }"
  >
    <v-icon icon="mdi-car" title="Наша машина" />
  </v-sheet>
</template>

<style lang="scss"></style>
