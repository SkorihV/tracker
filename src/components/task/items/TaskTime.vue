<script setup>
import {defineProps} from 'vue'

const props = defineProps({
  task: Object
})

defineOptions({name:'TaskTime'})
</script>

<template>
  <span class="mono font-weight-bold">{{ task.time }}</span>
</template>

<style lang="scss" scoped>

</style>