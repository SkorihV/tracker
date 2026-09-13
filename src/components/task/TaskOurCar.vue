<script setup>
import {useAppStore} from "../../store.js"
import {statusHex, statusTextHex} from "../../statusColors.js"
const props = defineProps({
  task: Object
})

const store = useAppStore()

defineOptions({name:'TaskOurCar'})
</script>

<template>
  <v-sheet
    v-if="props.task?.ourCar"
    height="100%"
    width="100%"
    min-width="60px"
    class="pa-2 our-car-cell d-flex align-center justify-center"
    :style="{ backgroundColor: statusHex(store.settings.ourCarColor), color: statusTextHex(store.settings.ourCarColor) }"
  >
    <v-icon icon="mdi-car" title="Наша машина" />
  </v-sheet>
</template>

<style lang="scss">

</style>