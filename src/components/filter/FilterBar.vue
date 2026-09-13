<script setup>
import { useAppStore } from "../../store.js"
import { dateRule } from "../../dateRules.js"

const store = useAppStore()

function resetFilters() {
  store.filter = { dateFrom: "", dateTo: "", tags: [], client: "", user: "", search: "" }
  store.refreshQuery()
}

function clearDates() {
  store.filter.dateFrom = ""
  store.filter.dateTo = ""
  store.refreshQuery()
}
</script>

<template>
  <div class="d-flex align-center ga-2 flex-wrap pb-3">
    <span class="text-body-2 text-medium-emphasis">С</span>
    <v-text-field
      v-model="store.filter.dateFrom"
      placeholder="дд.мм.гггг"
      v-maska="'##.##.####'"
      :rules="[dateRule]"
      density="compact"
      variant="outlined"
      hide-details
      style="max-width: 140px"
      @keydown.enter="store.refreshQuery()"
    />
    <span class="text-body-2 text-medium-emphasis">По</span>
    <v-text-field
      v-model="store.filter.dateTo"
      placeholder="дд.мм.гггг"
      v-maska="'##.##.####'"
      :rules="[dateRule]"
      density="compact"
      variant="outlined"
      hide-details
      style="max-width: 140px"
      @keydown.enter="store.refreshQuery()"
    />

    <v-autocomplete
      v-model="store.filter.user"
      :items="store.users.map((u) => ({ title: u, value: u }))"
      label="Пользователь"
      density="compact"
      variant="outlined"
      hide-details
      clearable
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
      style="max-width: 200px"
      @update:model-value="store.refreshQuery()"
    />

    <v-btn variant="outlined" size="small" @click="resetFilters">Сбросить</v-btn>
    <v-btn variant="text" size="small" @click="clearDates">Очистить даты</v-btn>
  </div>
</template>