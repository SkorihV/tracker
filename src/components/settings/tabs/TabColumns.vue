<script setup>
import { useColumnsStore } from "../../../stores/columns.js"

const columnsStore = useColumnsStore()

function moveUp(i) {
  if (i > 0) columnsStore.move(i, i - 1)
}

function moveDown(i) {
  if (i < columnsStore.columns.length - 1) columnsStore.move(i, i + 1)
}

function setVisible(key, visible) {
  columnsStore.setVisible(key, visible)
}
</script>

<template>
  <v-list density="compact" class="columns-tab">
    <v-list-item v-for="(col, i) in columnsStore.columns" :key="col.key">
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
          :disabled="i === columnsStore.columns.length - 1"
          title="Вниз"
          @click="moveDown(i)"
        >
          <v-icon>mdi-chevron-down</v-icon>
        </v-btn>
      </div>
    </v-list-item>
  </v-list>
  <v-btn variant="tonal" @click="columnsStore.reset()">По умолчанию</v-btn>
</template>
