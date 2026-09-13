<script setup>
import HeaderFieldGroup from "./HeaderFieldGroup.vue"
import HeaderBtnSearch from "./HeaderBtnSearch.vue"
import {useAppStore} from "../../store.js"
import HeaderFieldSearch from "./HeaderFieldSearch.vue"
const emits = defineEmits(['newTask'])

const store = useAppStore()

defineOptions({name: 'HeaderLayout'})
</script>

<template>
  <v-container fluid class="border-b pt-2 pb-1 px-4 d-flex justify-center align-center ga-4">
    <header-field-group/>
    <header-field-search/>
    <header-btn-search/>
    <v-spacer />
    <v-btn :color="store.settings.accentColor" height="90%" variant="flat" density="compact" prepend-icon="systemIcons:iconPlus" @click="emits('newTask')">
      Новая задача
    </v-btn>
    <v-btn variant="tonal" height="90%" density="compact" prepend-icon="systemIcons:iconCog" @click="store.showSettings = true">
      Настройки
    </v-btn>
  </v-container>
</template>

<style scoped lang="scss">

</style>