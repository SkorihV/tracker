<script setup>
import { defineProps } from "vue"

defineProps({
  task: Object,
})

defineOptions({ name: "TaskEnd" })
</script>

<template>
  <span class="mono">{{ task.end }}</span>
</template>

<style lang="scss" scoped></style>
