<script setup>
import { ref } from "vue";

const props = defineProps({
  kind: { type: String, required: true },
  title: { type: String, required: true },
  items: { type: Array, default: () => [] },
});

const emit = defineEmits(["action"]);

const newName = ref("");
const editingKey = ref(null);
const editName = ref("");

function keyOf(item) {
  return props.kind === "user" ? item : String(item.id);
}
function nameOf(item) {
  return typeof item === "string" ? item : item.name;
}

function startEdit(key, name) {
  editingKey.value = key;
  editName.value = name;
}

function cancelEdit() {
  editingKey.value = null;
}

function saveEdit(key, oldName) {
  const val = editName.value.trim();
  if (val && val !== oldName) {
    emit("action", { action: "rename", id: key, name: val, old: oldName });
  }
  cancelEdit();
}

function add() {
  const val = newName.value.trim();
  if (!val) return;
  emit("action", { action: "add", name: val });
  newName.value = "";
}

function remove(item) {
  const title = props.title;
  if (!window.confirm(`Удалить «${nameOf(item)}» из ${title.toLowerCase()}?`)) return;
  emit("action", { action: "remove", id: keyOf(item), name: nameOf(item) });
}
</script>

<template>
  <div>
    <v-list
      variant="outlined"
      density="compact"
      class="mb-3"
      style="max-height: 280px; overflow-y: auto"
    >
      <v-list-item v-if="!items.length" class="text-medium-emphasis text-caption">Нет элементов</v-list-item>
      <v-list-item v-for="item in items" :key="keyOf(item)" class="px-2">
        <template v-if="editingKey === keyOf(item)">
          <div class="d-flex align-center ga-2">
            <v-text-field
              v-model="editName"
              density="compact"
              variant="outlined"
              hide-details
              @keydown.enter="saveEdit(keyOf(item), nameOf(item))"
              @keydown.esc="cancelEdit"
            />
            <v-btn icon aria-label="Сохранить" variant="text" size="small" @click="saveEdit(keyOf(item), nameOf(item))">
              <v-icon icon="systemIcons:iconCheck" />
            </v-btn>
            <v-btn icon aria-label="Отмена" variant="text" size="small" @click="cancelEdit">
              <v-icon icon="systemIcons:iconClose" />
            </v-btn>
          </div>
        </template>
        <template v-else>
          <div class="d-flex align-center">
            <span class="flex-grow-1 text-truncate">{{ nameOf(item) }}</span>
            <v-btn icon aria-label="Редактировать" variant="text" size="small" @click="startEdit(keyOf(item), nameOf(item))">
              <v-icon icon="systemIcons:iconEdit" />
            </v-btn>
            <v-btn icon aria-label="Удалить" variant="text" size="small" color="error" @click="remove(item)">
              <v-icon icon="systemIcons:iconTrash" />
            </v-btn>
          </div>
        </template>
      </v-list-item>
    </v-list>

    <div class="d-flex ga-2">
      <v-text-field
        v-model="newName"
        placeholder="Новое имя…"
        density="compact"
        variant="outlined"
        hide-details
        @keydown.enter="add"
      />
      <v-btn variant="tonal" @click="add">Добавить</v-btn>
    </div>
  </div>
</template>