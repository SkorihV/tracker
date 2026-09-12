<script setup>
import { ref } from "vue";

const props = defineProps({
  text: { type: String, default: "" },
});

const emit = defineEmits(["save", "close"]);

const error = ref("");
const localText = ref(props.text);

const LINE_RE =
  /^\d{2}\.\d{2}\.\d{4} \d{2}:\d{2}(?:\s*[—–-]\s*(?:\d{2}\.\d{2}\.\d{4} \d{2}:\d{2}|\d{2}:\d{2}|открыт)?)?$/i;

function validate() {
  error.value = "";
  const lines = localText.value
    .split("\n")
    .map((l) => l.trim())
    .filter(Boolean);
  if (!lines.length) {
    error.value = "Должен быть хотя бы один диапазон";
    return null;
  }
  for (const line of lines) {
    if (!LINE_RE.test(line)) {
      error.value = `Неверный формат строки: «${line}»`;
      return null;
    }
  }
  if (lines.length > 1) {
    for (let i = 0; i < lines.length - 1; i++) {
      if (!lines[i].includes("—") && !lines[i].includes("–") && !lines[i].includes("-")) {
        error.value = "Открытым может быть только последний диапазон";
        return null;
      }
    }
  }
  return lines;
}

function save() {
  const lines = validate();
  if (lines) emit("save", lines);
}
</script>

<template>
  <v-dialog :model-value="true" width="720" height="65vh">
    <v-card class="modal-card">
      <v-toolbar v-dialog-drag density="compact" color="primary">
        <v-toolbar-title>Временные диапазоны</v-toolbar-title>
        <v-spacer />
        <v-btn icon variant="text" title="Закрыть" @click="emit('close')">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pa-4 flex-grow-1 d-flex flex-column">
        <div class="text-caption text-medium-emphasis mb-2">
          По строке: «дд.мм.гггг чч:мм — дд.мм.гггг чч:мм» (или просто «дд.мм.гггг чч:мм» — открытый).
          Завершение можно не заполнять.
        </div>
        <v-textarea
          v-model="localText"
          rows="12"
          variant="outlined"
          density="compact"
          class="ranges-textarea flex-grow-1"
          hide-details
          autofocus
          @keydown.esc="emit('close')"
        />
        <v-alert v-if="error" type="error" density="compact" class="mt-3">
          {{ error }}
        </v-alert>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="emit('close')">Отмена</v-btn>
        <v-btn color="primary" variant="flat" @click="save">Сохранить</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<style scoped>
.ranges-textarea :deep(textarea) {
  font-family: "Cascadia Mono", "Fira Code", Consolas, monospace !important;
  font-size: 13px;
  line-height: 1.6;
  tab-size: 4;
}
</style>