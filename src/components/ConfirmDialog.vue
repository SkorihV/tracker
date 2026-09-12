<script setup>
import { onMounted, onBeforeUnmount } from "vue";

defineProps({
  title: { type: String, default: "Подтвердите действие" },
  message: { type: String, default: "" },
  confirmText: { type: String, default: "Удалить" },
  color: { type: String, default: "error" },
  maxWidth: { type: Number, default: 420 },
});

const emit = defineEmits(["confirm", "close"]);

function onKeydown(e) {
  const tag = (e.target.tagName || "").toLowerCase();
  if (["input", "textarea", "select", "button"].includes(tag)) return;
  if (e.key === "Enter" && !e.repeat) emit("confirm");
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <v-dialog
    model-value
    :max-width="maxWidth"
    @update:model-value="(v) => !v && emit('close')"
  >
    <v-card class="modal-card">
      <v-toolbar v-dialog-drag density="compact" :color="color">
        <v-toolbar-title>{{ title }}</v-toolbar-title>
        <v-spacer />
        <v-btn icon variant="text" title="Закрыть" @click="emit('close')">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pt-5">
        {{ message }}
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="emit('close')">Отмена</v-btn>
        <v-btn :color="color" variant="flat" @click="emit('confirm')">{{ confirmText }}</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>