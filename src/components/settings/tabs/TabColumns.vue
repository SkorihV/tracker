<script setup>
import { useAppStore } from "../../../store.js"

const store = useAppStore()

function moveUp(i) {
  if (i > 0) store.moveColumn(i, i - 1)
}

function moveDown(i) {
  if (i < store.columns.length - 1) store.moveColumn(i, i + 1)
}

function setVisible(key, visible) {
  store.setColumnVisible(key, visible)
}
</script>

<template>
  <v-list density="compact" class="columns-tab">
    <v-list-item v-for="(col, i) in store.columns" :key="col.key">
      <div class="d-flex align-center">
        <v-checkbox
          :model-value="col.visible"
          density="compact"
          hide-details
          color="primary"
          class="mr-2"
          :label="col.label"
          @update:model-value="setVisible(col.key, $event)"
        />
        <v-spacer />
        <v-btn icon size="small" variant="text" :disabled="i === 0" title="Вверх" @click="moveUp(i)">
          <v-icon>mdi-chevron-up</v-icon>
        </v-btn>
        <v-btn
          icon
          size="small"
          variant="text"
          :disabled="i === store.columns.length - 1"
          title="Вниз"
          @click="moveDown(i)"
        >
          <v-icon>mdi-chevron-down</v-icon>
        </v-btn>
      </div>
    </v-list-item>
  </v-list>
  <v-btn variant="tonal" @click="store.resetColumns()">По умолчанию</v-btn>
</template>