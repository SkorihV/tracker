<script setup>
import { useAppStore } from "../../stores/app.js"
import { useDictionariesStore } from "../../stores/dictionaries.js"
import { useTasksStore } from "../../stores/tasks.js"
import { today } from "../../reportFilter.js"
import BaseDateField from "../base/BaseDateField.vue"

const appStore = useAppStore()
const dicts = useDictionariesStore()
const tasksStore = useTasksStore()

function resetFilters() {
  tasksStore.filter = { dateFrom: today(), dateTo: today(), tags: [], client: "", user: "", search: "" }
  appStore.refreshQuery()
}
</script>

<template>
  <div class="d-flex align-center ga-2 flex-wrap pb-3">
    <base-date-field
      v-model="tasksStore.filter.dateFrom"
      label="С"
      clearable
      clear-to-today
      style="max-width: 180px"
      @update:model-value="appStore.refreshQuery()"
    />
    <base-date-field
      v-model="tasksStore.filter.dateTo"
      label="По"
      clearable
      clear-to-today
      style="max-width: 180px"
      @update:model-value="appStore.refreshQuery()"
    />

    <v-autocomplete
      v-model="tasksStore.filter.user"
      :items="dicts.users.map((u) => ({ title: u, value: u }))"
      label="Пользователь"
      density="compact"
      variant="outlined"
      hide-details
      clearable
      prepend-inner-icon="systemIcons:iconUser"
      style="max-width: 200px"
      @update:model-value="appStore.refreshQuery()"
    />

    <v-autocomplete
      v-model="tasksStore.filter.tags"
      :items="dicts.tags.map((t) => ({ title: t.name, value: t.name }))"
      label="Теги"
      density="compact"
      variant="outlined"
      hide-details
      clearable
      multiple
      chips
      prepend-inner-icon="mdi-tag-outline"
      style="max-width: 240px"
      @update:model-value="appStore.refreshQuery()"
    />

    <v-autocomplete
      v-model="tasksStore.filter.client"
      :items="dicts.clients.map((c) => ({ title: c.name, value: c.name }))"
      label="Клиент"
      density="compact"
      variant="outlined"
      hide-details
      clearable
      prepend-inner-icon="systemIcons:iconClient"
      style="max-width: 200px"
      @update:model-value="appStore.refreshQuery()"
    />

    <v-btn variant="outlined" @click="resetFilters">Сбросить</v-btn>
  </div>
</template>
