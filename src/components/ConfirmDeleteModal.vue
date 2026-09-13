<script setup>
import { computed } from "vue"
import ConfirmDialog from "./ConfirmDialog.vue"

const props = defineProps({
  ids: { type: Array, default: () => [] },
})

const emit = defineEmits(["confirm", "close"])

const count = computed(() => props.ids.length)

const title = computed(() =>
  count.value === 1 ? "Удалить задачу" : `Удалить выбранные задачи (${count.value})`
)

const message = computed(() =>
  count.value === 1
    ? "Задача будет удалена безвозвратно. Продолжить?"
    : `Будет удалено безвозвратно задач: ${count.value}. Продолжить?`
)
</script>

<template>
  <ConfirmDialog
    :title="title"
    :message="message"
    @confirm="emit('confirm')"
    @close="emit('close')"
  />
</template>