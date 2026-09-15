<script setup>
import {defineProps} from 'vue'

const props = defineProps({
  task: Object
})

defineOptions({name:'TaskStart'})
</script>

<template>
  <span class="mono">{{ task.start }}</span>
</template>

<style lang="scss" scoped>

</style>