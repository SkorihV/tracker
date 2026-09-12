<script setup>
import {useAppStore} from "../../store.js";
const store = useAppStore()

defineOptions({name:'HeaderFieldSearch'})
</script>

<template>
  <v-text-field
      ref="searchRef"
      v-model="store.filter.search"
      placeholder="Поиск (ID, заявка, комментарий…)"
      variant="outlined"
      density="compact"
      hide-details
      clearable
      min-width="150px"
      prepend-inner-icon="systemIcons:iconSearch"
      @keydown.enter="store.refreshQuery()"
      @click:clear="store.refreshQuery()"
  />
</template>

<style scoped lang="scss">

</style>