<script setup>
import { defineModel } from 'vue'
import { dateRule } from "../dateRules.js"

const dateFrom = defineModel('dateFrom', { type: String, default: '' })
const dateTo = defineModel('dateTo', { type: String, default: '' })
const format = defineModel('format', { type: String, default: 'txt' })

defineProps({
  emptyHint: { type: String, default: "" },
})

const formats = ["txt", "md", "csv", "xlsx"]

defineOptions({name:'ReportPeriodFields'})
</script>

<template>
  <div class="d-flex ga-4 mb-3 flex-wrap">
    <v-text-field
      v-model="dateFrom"
      label="Дата с (дд.мм.гггг)"
      :placeholder="emptyHint"
      v-maska="'##.##.####'"
      :rules="[dateRule]"
      density="compact"
      variant="outlined"
      hide-details
      style="max-width: 220px"
    />
    <v-text-field
      v-model="dateTo"
      label="Дата по (дд.мм.гггг)"
      :placeholder="emptyHint"
      v-maska="'##.##.####'"
      :rules="[dateRule]"
      density="compact"
      variant="outlined"
      hide-details
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