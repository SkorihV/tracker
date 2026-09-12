<script setup>
import {useAppStore} from "../../store.js";

const store = useAppStore()
defineOptions({name:'HeaderBtnSearch'})
</script>

<template>
  <v-btn variant="tonal" height="90%" density="compact" @click="store.refreshQuery()">Найти</v-btn>
</template>

<style scoped lang="scss">

</style>