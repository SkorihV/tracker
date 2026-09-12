<script setup>
import {useAppStore} from "../../store.js";
const store = useAppStore()

function onGroupingChange(value) {
  store.applySettings(store.settings.username, value);
}

defineOptions({name:'HeaderFieldGroup'})
</script>

<template>
  <v-select
      :model-value="store.settings.grouping"
      :items="[
          { title: 'Нет', value: 'none' },
          { title: 'По дням', value: 'day' },
          { title: 'По клиентам', value: 'client' },
        ]"
      label="Группировка"
      variant="outlined"
      density="compact"
      hide-details
      min-width="150px"
      @update:model-value="onGroupingChange"
  />
</template>

<style scoped lang="scss">

</style>