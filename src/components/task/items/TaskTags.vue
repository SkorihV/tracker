<script setup>
import {defineProps} from 'vue'

const props = defineProps({
  task: Object
})

defineOptions({name:'TaskTags'})
</script>

<template>
  {{ (task.tags || []).join(", ") }}
</template>

<style lang="scss" scoped>

</style>