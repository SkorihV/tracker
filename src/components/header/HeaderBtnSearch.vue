<script setup>
import { useAppStore } from "../../stores/app.js"

const appStore = useAppStore()
defineOptions({ name: "HeaderBtnSearch" })
</script>

<template>
  <v-btn variant="tonal" height="90%" density="compact" @click="appStore.refreshQuery()">Найти</v-btn>
</template>

<style scoped lang="scss"></style>
