<script setup>
import { defineModel } from 'vue'
import BaseDateField from "./BaseDateField.vue"

const dateFrom = defineModel('dateFrom', { type: String, default: '' })
const dateTo = defineModel('dateTo', { type: String, default: '' })
const format = defineModel('format', { type: String, default: 'xlsx' })

defineProps({
  emptyHint: { type: String, default: "" },
})

const formats = ["xlsx","txt", "md", "csv"]

defineOptions({name:'BasePeriodFields'})
</script>

<template>
  <div class="d-flex ga-4 mb-3 flex-wrap">
    <base-date-field
      v-model="dateFrom"
      label="Дата с"
      :placeholder="emptyHint"
      clearable
      style="max-width: 220px"
    />
    <base-date-field
      v-model="dateTo"
      label="Дата по"
      :placeholder="emptyHint"
      clearable
      style="max-width: 220px"
    />
    <v-autocomplete
      v-model="format"
      :items="formats.map((f) => ({ title: f.toUpperCase(), value: f }))"
      label="Формат файла"
      density="compact"
      variant="outlined"
      hide-details
      style="max-width: 180px"
    />
  </div>
</template>

<style lang="scss">

</style>