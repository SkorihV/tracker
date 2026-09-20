<script setup>
import { useAppStore } from "../../store.js"
import { today } from "../../reportFilter.js"
import BaseDateField from "../base/BaseDateField.vue"

const store = useAppStore()

function resetFilters() {
  store.filter = { dateFrom: today(), dateTo: today(), tags: [], client: "", user: "", search: "" }
  store.refreshQuery()
}
</script>

<template>
  <div class="d-flex align-center ga-2 flex-wrap pb-3">
    <base-date-field
      v-model="store.filter.dateFrom"
      label="С"
      clearable
      clear-to-today
      style="max-width: 180px"
      @update:model-value="store.refreshQuery()"
    />
    <base-date-field
      v-model="store.filter.dateTo"
      label="По"
      clearable
      clear-to-today
      style="max-width: 180px"
      @update:model-value="store.refreshQuery()"
    />

    <v-autocomplete
      v-model="store.filter.user"
      :items="store.users.map((u) => ({ title: u, value: u }))"
      label="Пользователь"
      density="compact"
      variant="outlined"
      hide-details
      clearable
      prepend-inner-icon="systemIcons:iconUser"
      style="max-width: 200px"
      @update:model-value="store.refreshQuery()"
    />

    <v-autocomplete
      v-model="store.filter.tags"
      :items="store.tags.map((t) => ({ title: t.name, value: t.name }))"
      label="Теги"
      density="compact"
      variant="outlined"
      hide-details
      clearable
      multiple
      chips
      prepend-inner-icon="mdi-tag-outline"
      style="max-width: 240px"
      @update:model-value="store.refreshQuery()"
    />

    <v-autocomplete
      v-model="store.filter.client"
      :items="store.clients.map((c) => ({ title: c.name, value: c.name }))"
      label="Клиент"
      density="compact"
      variant="outlined"
      hide-details
      clearable
      prepend-inner-icon="systemIcons:iconClient"
      style="max-width: 200px"
      @update:model-value="store.refreshQuery()"
    />

    <v-btn variant="outlined" @click="resetFilters">Сбросить</v-btn>
  </div>
</template>