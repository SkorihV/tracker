<script setup>
import { computed } from "vue"

const props = defineProps({
  title: { type: String, default: "" },
  maxWidth: { type: [Number, String], default: 600 },
  maxHeight: { type: [Number, String], default: "85vh" },
  width: { type: [Number, String], default: undefined },
  height: { type: [Number, String], default: undefined },
  bodyClass: { type: String, default: "pt-5" },
})

const emit = defineEmits(["close"])

defineOptions({name:'BaseModal'})
</script>

<template>
  <v-dialog
    model-value
    :max-width="maxWidth"
    :max-height="maxHeight"
    :width="width"
    :height="height"
    @update:model-value="(v) => !v && emit('close')"
  >
    <v-card class="modal-card">
      <v-toolbar v-dialog-drag density="compact" color="primary">
        <v-toolbar-title>{{ title }}</v-toolbar-title>
        <v-spacer />
        <v-btn icon variant="text" title="Закрыть" @click="emit('close')">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text :class="bodyClass">
        <slot />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <slot name="actions" />
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style lang="scss">

</style>