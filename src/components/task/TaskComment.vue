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
  max-width: 260px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>