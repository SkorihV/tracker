<script setup>
import { ref, nextTick } from "vue"
import BaseConfirmDialog from "../../base/BaseConfirmDialog.vue"

const props = defineProps({
  kind: { type: String, required: true },
  title: { type: String, required: true },
  items: { type: Array, default: () => [] },
})

const emit = defineEmits(["action"])

const newName = ref("")
const editingKey = ref(null)
const editName = ref("")
const pendingDelete = ref(null)
const pendingClear = ref(false)
const editInputs = {}

const placeholder = {
  user: "Новый пользователь…",
  tag: "Новый тег…",
  client: "Новый клиент…",
}[props.kind] || "Новое имя…"

function keyOf(item) {
  return props.kind === "user" ? item : String(item.id)
}
function nameOf(item) {
  return typeof item === "string" ? item : item.name
}

function startEdit(key, name) {
  editingKey.value = key
  editName.value = name
  nextTick(() => editInputs[key]?.focus())
}

function cancelEdit() {
  editingKey.value = null
}

function saveEdit(key, oldName) {
  const val = editName.value.trim()
  if (val && val !== oldName) {
    emit("action", { action: "rename", id: key, name: val, old: oldName })
  }
  cancelEdit()
}

function add() {
  const val = newName.value.trim()
  if (!val) return
  emit("action", { action: "add", name: val })
  newName.value = ""
}

function remove(item) {
  pendingDelete.value = { key: keyOf(item), name: nameOf(item), item }
}

function confirmRemove() {
  if (!pendingDelete.value) return
  const { key, name, item } = pendingDelete.value
  pendingDelete.value = null
  // backend remove_user ждёт имя; remove_tag/remove_client — числовой id
  const id = props.kind === "user" ? key : item.id
  emit("action", { action: "remove", id, name })
}

function clearAll() {
  pendingClear.value = true
}

function confirmClear() {
  pendingClear.value = false
  emit("action", { action: "clear" })
}

function move(index, step) {
  const to = index + step
  if (to < 0 || to >= props.items.length) return
  emit("action", { action: "move", from: index, to })
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
      <v-list-item v-for="(item, i) in items" :key="keyOf(item)" class="px-2">
        <template v-if="editingKey === keyOf(item)">
          <div class="d-flex align-center ga-2">
            <v-text-field
              :ref="(el) => { if (el) editInputs[keyOf(item)] = el; }"
              v-model="editName"
              density="compact"
              variant="outlined"
              hide-details
              @keydown.enter="saveEdit(keyOf(item), nameOf(item))"
              @keydown.esc.stop="cancelEdit"
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
            <v-btn icon aria-label="Вверх" variant="text" size="small" :disabled="i === 0" @click="move(i, -1)">
              <v-icon>mdi-chevron-up</v-icon>
            </v-btn>
            <v-btn icon aria-label="Вниз" variant="text" size="small" :disabled="i === items.length - 1" @click="move(i, 1)">
              <v-icon>mdi-chevron-down</v-icon>
            </v-btn>
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
        :placeholder="placeholder"
        density="compact"
        variant="outlined"
        hide-details
        @keydown.enter="add"
      />
      <v-btn variant="tonal" @click="add">Добавить</v-btn>
      <v-btn variant="tonal" color="error" :disabled="!items.length" @click="clearAll">
        Удалить все
      </v-btn>
    </div>

    <BaseConfirmDialog
      v-if="pendingDelete"
      :title="`Удалить «${pendingDelete.name}»?`"
      :message="`«${pendingDelete.name}» будет удален безвозвратно из ${props.title.toLowerCase()}. Продолжить?`"
      @confirm="confirmRemove"
      @close="pendingDelete = null"
    />
    <BaseConfirmDialog
      v-if="pendingClear"
      title="Удалить все элементы?"
      :message="`Все элементы будут удалены безвозвратно из ${props.title.toLowerCase()}. Продолжить?`"
      @confirm="confirmClear"
      @close="pendingClear = false"
    />
  </div>
</template>