<script setup>
import { ref, reactive } from "vue";
import { statusHex } from "../../../statusColors.js";

const emit = defineEmits(["action"]);
const props = defineProps({
  items: { type: Array, default: () => [] },
});

const newName = ref("");
const newColor = ref("");
const newMenu = ref(false);
const editingKey = ref(null);
const editName = ref("");
const menus = reactive({});
const picker = reactive({});

function nameOf(item) {
  return item.name;
}

function colorOf(item) {
  return item.color || "";
}

function swatchStyle(color) {
  const c = statusHex(color);
  return c ? `background-color: ${c};` : "";
}

function currentHex(item) {
  return statusHex(colorOf(item)) || "#1E88E5";
}

function openColorMenu(item) {
  picker[item.id] = currentHex(item);
  menus[item.id] = true;
}

function applyColor(item) {
  const c = (picker[item.id] || "").trim();
  if (c !== colorOf(item)) {
    emit("action", { action: "color", id: item.id, color: c });
  }
  menus[item.id] = false;
}

function startEdit(item) {
  editingKey.value = item.id;
  editName.value = item.name;
}

function cancelEdit() {
  editingKey.value = null;
}

function saveEdit(item) {
  const val = editName.value.trim();
  if (val && val !== nameOf(item)) {
    emit("action", { action: "rename", old: nameOf(item), name: val });
  }
  cancelEdit();
}

function applyNewColor() {
  newColor.value = (picker.new || "").trim();
  newMenu.value = false;
}

function add() {
  const val = newName.value.trim();
  if (!val) return;
  emit("action", { action: "add", name: val, color: newColor.value });
  newName.value = "";
}

function remove(item) {
  if (!window.confirm(`Удалить статус «${nameOf(item)}»?`)) return;
  emit("action", { action: "remove", id: item.id });
}
</script>

<template>
  <div>
    <v-list variant="outlined" density="compact" class="mb-3" style="max-height: 260px; overflow-y: auto">
      <v-list-item v-if="!items.length" class="text-medium-emphasis text-caption">
        Статус не задан. Добавьте первый (например «Новая», «В работе», «Завершена»).
      </v-list-item>
      <v-list-item v-for="item in items" :key="item.id" class="px-2">
          <div class="d-flex align-center ga-2">
            <v-menu v-model="menus[item.id]" :close-on-content-click="false" location="bottom start">
              <template #activator="{ props: menuProps }">
                <v-btn
                    v-bind="menuProps"
                    icon
                    size="small"
                    variant="flat"
                    :aria-label="nameOf(item)"
                    :style="swatchStyle(colorOf(item))"
                    @click="openColorMenu(item)"
                />
              </template>
              <v-sheet class="pa-2 d-flex flex-column ga-2">
                <v-color-picker
                    v-model="picker[item.id]"
                    mode="hex"
                    hide-inputs
                    :width="280"
                />
                <v-btn color="primary" variant="flat" density="compact" @click="applyColor(item)">
                  Применить
                </v-btn>
              </v-sheet>
            </v-menu>

            <v-text-field
                v-if="editingKey === item.id"
              v-model="editName"
              density="compact"
              variant="outlined"
              hide-details
              class="flex-grow-1"
              @keydown.enter="saveEdit(item)"
              @keydown.esc="cancelEdit"
            />
            <span v-else class="flex-grow-1 text-truncate px-2">{{ nameOf(item) }}</span>
            <template v-if="editingKey === item.id">
              <v-btn icon aria-label="Сохранить" variant="text" size="small" @click="saveEdit(item)">
                <v-icon icon="systemIcons:iconCheck" />
              </v-btn>
              <v-btn icon aria-label="Отмена" variant="text" size="small" @click="cancelEdit">
                <v-icon icon="systemIcons:iconClose" />
              </v-btn>
            </template>
            <template v-else>
              <v-btn icon aria-label="Редактировать" variant="text" size="small" @click="startEdit(item)">
                <v-icon icon="systemIcons:iconEdit" />
              </v-btn>
              <v-btn icon aria-label="Удалить" variant="text" size="small" color="error" @click="remove(item)">
                <v-icon icon="systemIcons:iconTrash" />
              </v-btn>
            </template>
          </div>
      </v-list-item>
    </v-list>

    <div class="d-flex ga-2">
      <v-text-field
        v-model="newName"
        placeholder="Новый статус…"
        density="compact"
        variant="outlined"
        hide-details
        class="flex-grow-1"
        @keydown.enter="add"
      />
      <v-menu v-model="newMenu" :close-on-content-click="false" location="bottom start">
        <template #activator="{ props: menuProps }">
          <v-btn
            v-bind="menuProps"
            variant="tonal"
            aria-label="Цвет нового статуса"
            :style="swatchStyle(newColor)"
            @click="picker.new = statusHex(newColor) || '#1E88E5'; newMenu = true"
          />
        </template>
        <v-sheet class="pa-2 d-flex flex-column ga-2">
          <v-color-picker
            v-model="picker.new"
            mode="hex"
            hide-inputs
            :width="280"
          />
          <v-btn color="primary" variant="flat" density="compact" @click="applyNewColor">
            ОК
          </v-btn>
        </v-sheet>
      </v-menu>
      <v-btn variant="tonal" @click="add">Добавить</v-btn>
    </div>
  </div>
</template>