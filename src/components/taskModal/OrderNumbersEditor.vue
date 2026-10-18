<script setup>
import { ref, nextTick } from "vue"
import BaseConfirmDialog from "../base/BaseConfirmDialog.vue"
import { useAppStore } from "../../store.js"

const store = useAppStore()

const orders = defineModel("orders", { type: Array, default: () => [] })
const orderInput = defineModel("orderInput", { type: String, default: "" })
const editingIndex = ref(null)
const editName = ref("")
const editInputs = {}
const pendingClear = ref(false)

function splitOrders(s) {
  return s.split(/[,\s]+/).map((x) => x.trim()).filter(Boolean)
}

function add() {
  const parts = splitOrders(orderInput.value)
  if (!parts.length) return
  for (const val of parts) {
    if (!orders.value.includes(val)) orders.value.push(val)
  }
  orderInput.value = ""
}

function remove(i) {
  orders.value.splice(i, 1)
  if (editingIndex.value === i) cancelEdit()
  else if (editingIndex.value !== null && editingIndex.value > i) editingIndex.value -= 1
}

function startEdit(i) {
  editingIndex.value = i
  editName.value = orders.value[i]
  nextTick(() => editInputs[i]?.focus())
}

function cancelEdit() {
  editingIndex.value = null
}

function editDirty(i) {
  const v = editName.value.trim()
  return !!v && v !== orders.value[i]
}

function saveEdit(i) {
  const val = editName.value.trim()
  const dup = orders.value.some((x, j) => j !== i && x === val)
  if (val && val !== orders.value[i] && !dup) {
    orders.value[i] = val
  }
  cancelEdit()
}

defineExpose({ add })
</script>

<template>
  <div>
    <v-text-field
      v-model="orderInput"
      label="Номер заявки (заказ)"
      density="compact"
      variant="outlined"
      prepend-inner-icon="mdi-clipboard-text-outline"
      append-inner-icon="systemIcons:iconPlus"
      hint="Enter, «+» или «Сохранить» — добавить в список; можно перечислить несколько номеров через запятую или пробел"
      persistent-hint
      class="mb-3"
      @keydown.enter.prevent="add"
      @click:append-inner="add"
    />
    <v-list
      v-if="orders.length"
      variant="outlined"
      density="compact"
      class="order-list mb-1"
    >
      <v-list-item density="compact" v-for="(o, i) in orders" :key="i" class="px-2">
        <template v-if="editingIndex === i">
          <div class="d-flex align-center ga-2">
            <v-text-field
              :ref="(el) => { if (el) editInputs[i] = el; }"
              v-model="editName"
              density="compact"
              class="order-item"
              variant="outlined"
              hide-details
              tabindex="-1"
              @keydown.enter="saveEdit(i)"
              @keydown.esc.stop="cancelEdit"
              @blur="cancelEdit"
            />
            <v-btn
                icon="systemIcons:iconCheck"
                aria-label="Сохранить"
                :variant="editDirty(i) ? 'flat' : 'text'"
                size="x-small"
                :disabled="!editDirty(i)"
                :color="editDirty(i) ? store.settings.accentColor : undefined"
                @mousedown.prevent
                @click="saveEdit(i)">
            </v-btn>
            <v-btn
                icon="systemIcons:iconClose"
                aria-label="Отмена"
                variant="text"
                size="x-small"
                @mousedown.prevent
                @click="cancelEdit">
            </v-btn>
          </div>
        </template>
        <template v-else>
          <div class="d-flex align-center">
            <span class="ml-4 flex-grow-1 text-truncate">{{ o }}</span>
            <v-btn
                icon="systemIcons:iconEdit"
                aria-label="Редактировать"
                variant="text"
                size="x-small"
                tabindex="-1"
                @click="startEdit(i)">
            </v-btn>
            <v-btn icon="systemIcons:iconTrash"
                   aria-label="Удалить"
                   variant="text"
                   size="x-small"
                   tabindex="-1"
                   color="error" @click="remove(i)">
            </v-btn>
          </div>
        </template>
      </v-list-item>
    </v-list>
    <div v-if="orders.length" class="d-flex justify-end mb-3">
      <v-btn variant="tonal" color="error" tabindex="-1" size="small" @click="pendingClear = true">
        Удалить все
      </v-btn>
    </div>

    <BaseConfirmDialog
      v-if="pendingClear"
      title="Удалить все номера заявок?"
      message="Все номера заявок будут удалены из задачи. Продолжить?"
      @confirm="orders = []; pendingClear = false"
      @close="pendingClear = false"
    />
  </div>
</template>

<style lang="scss" scoped>
.order-list {
  max-height: 180px;
  overflow-y: auto;
  .order-item {
    :deep(.v-field__input) {
      min-height: 16px;
      padding-block: 4px;

    }
  }
}
</style>
