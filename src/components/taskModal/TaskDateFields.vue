<script setup>
import { dateTimeRule } from "../../dateRules.js"

const start = defineModel("start", { type: String, default: "" })
const end = defineModel("end", { type: String, default: "" })
</script>

<template>
  <div class="d-flex ga-4 mb-3">
    <v-text-field
      v-model="start"
      label="Начало"
      width="50%"
      placeholder="дд.мм.гггг мм:чч"
      v-maska="'##.##.#### ##:##'"
      :rules="[dateTimeRule]"
      density="compact"
      variant="outlined"
      prepend-inner-icon="mdi-clock-start"
      class="flex-grow-1"
    />
    <v-text-field
      v-model="end"
      label="Завершение"
      width="50%"
      placeholder="дд.мм.гггг мм:чч"
      v-maska="'##.##.#### ##:##'"
      :rules="[dateTimeRule]"
      density="compact"
      variant="outlined"
      clearable
      prepend-inner-icon="mdi-clock-end"
      class="flex-grow-1"
    />
  </div>
</template>
