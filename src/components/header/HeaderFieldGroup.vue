<script setup>
import { useSettingsStore } from "../../stores/settings.js"
const settingsStore = useSettingsStore()

function onGroupingChange(value) {
  settingsStore.applySettings(settingsStore.settings.username, value)
}

defineOptions({ name: "HeaderFieldGroup" })
</script>

<template>
  <v-select
    :model-value="settingsStore.settings.grouping"
    :items="[
      { title: 'Нет', value: 'none' },
      { title: 'По дням', value: 'day' },
      { title: 'По клиентам', value: 'client' },
    ]"
    label="Группировка"
    variant="outlined"
    density="compact"
    hide-details
    min-width="150px"
    @update:model-value="onGroupingChange"
  />
</template>

<style scoped lang="scss"></style>
