<script setup>
import { useAppStore } from "../../stores/app.js"
import { useTasksStore } from "../../stores/tasks.js"
const appStore = useAppStore()
const tasksStore = useTasksStore()

defineOptions({ name: "HeaderFieldSearch" })
</script>

<template>
  <v-text-field
    ref="searchRef"
    v-model="tasksStore.filter.search"
    placeholder="Поиск (ID, заявка, комментарий…)"
    variant="outlined"
    density="compact"
    hide-details
    clearable
    min-width="150px"
    prepend-inner-icon="systemIcons:iconSearch"
    @keydown.enter="appStore.refreshQuery()"
    @click:clear="appStore.refreshQuery()"
  />
</template>

<style scoped lang="scss"></style>
