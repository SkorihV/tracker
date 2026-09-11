<script setup>
import { ref } from "vue";
import SvgIcon from "../icons/SvgIcon.vue";

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
  return props.kind === "user" ? item : item.id;
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
  const key = keyOf(item);
  const title = props.title;
  if (!window.confirm(`Удалить «${nameOf(item)}» из ${title.toLowerCase()}?`)) return;
  emit("action", { action: "remove", id: key, name: nameOf(item) });
}
</script>

<template>
  <div class="entity-panel">
    <h3>{{ title }}</h3>
    <div class="entity-list">
      <div v-if="!items.length" class="empty" style="padding: 8px">Нет элементов</div>
      <div v-for="item in items" :key="keyOf(item)" class="entity-row">
        <template v-if="editingKey === keyOf(item)">
          <input
            v-model="editName"
            style="flex: 1; min-width: 0"
            @keydown.enter="saveEdit(keyOf(item), nameOf(item))"
            @keydown.esc="cancelEdit"
          />
          <button class="small" @click="saveEdit(keyOf(item), nameOf(item))"><SvgIcon name="check" /></button>
          <button class="small" @click="cancelEdit"><SvgIcon name="close" /></button>
        </template>
        <template v-else>
          <span style="flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis">
            {{ nameOf(item) }}
          </span>
          <button class="small with-icon" @click="startEdit(keyOf(item), nameOf(item))"><SvgIcon name="edit" /></button>
          <button class="small danger with-icon" @click="remove(item)"><SvgIcon name="trash" /></button>
        </template>
      </div>
    </div>
    <div class="entity-add">
      <input
        v-model="newName"
        placeholder="Новое имя…"
        @keydown.enter="add"
      />
      <button class="small" @click="add">Добавить</button>
    </div>
  </div>
</template>