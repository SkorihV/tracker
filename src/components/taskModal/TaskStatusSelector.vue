<script setup>
import { computed } from "vue"
import { useAppStore } from "../../store.js"

const store = useAppStore()

const status = defineModel("status", { type: String, default: "" })
const ourCar = defineModel("ourCar", { type: Boolean, default: false })

const statusColor = computed(() => {
  return store.statuses.find((s) => s.name === status.value)?.color ?? null
})
</script>

<template>
  <div class="d-flex ga-4">
    <v-sheet
        class="rounded-circle"
        width="40px"
        height="40px"
        border
        :color="statusColor"
      >
      </v-sheet>
    <v-autocomplete
      v-model="status"
      :items="store.statuses.map((s) => ({ title: s.name, value: s.name }))"
      label="Статус"
      density="compact"
      variant="outlined"
      clearable
      prepend-inner-icon="mdi-flag-outline"
      class="mb-3"
    />
    <div>
      <v-checkbox
        v-model="ourCar"
        label="Наша машина"
        density="compact"
        hide-details
        color="primary"
      />
    </div>
  </div>
</template>
