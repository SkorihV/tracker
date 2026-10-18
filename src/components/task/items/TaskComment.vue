<script setup>
import {defineProps} from 'vue'

const props = defineProps({
  task: Object
})

defineOptions({name:'TaskComment'})
</script>

<template>
  <div class="comment-cell">{{ task.comment }}</div>
</template>

<style lang="scss" scoped>

.comment-cell {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  max-width: 500px;
}
</style>