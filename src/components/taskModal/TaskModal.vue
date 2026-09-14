<script setup>
import { reactive, computed, ref, nextTick } from "vue"
import { useAppStore } from "../../store.js"
import RangesModal from "../RangesModal.vue"
import ConfirmDialog from "../ConfirmDialog.vue"
import BaseModal from "../BaseModal.vue"
import { dateTimeRule, isDateTimeValid } from "../../dateRules.js"

const store = useAppStore()

const props = defineProps({
  draft: { type: Object, required: true },
})

const emit = defineEmits(["save", "close"])

function nowLabel() {
  const d = new Date()
  const p = (n) => String(n).padStart(2, "0")
  return `${p(d.getDate())}.${p(d.getMonth() + 1)}.${d.getFullYear()} ${p(d.getHours())}:${p(d.getMinutes())}`
}

const isEdit = props.draft.mode === "edit"

const form = reactive({
  mode: props.draft.mode,
  taskId: props.draft.taskId || "",
  user: props.draft.user || store.settings.username || "",
  orders: [...(props.draft.orders || [])],
  orderInput: "",
  tags: props.draft.tags || [],
  customStatus: props.draft.customStatus || "",
  ourCar: props.draft.ourCar || false,
  client: props.draft.client || "",
  comment: props.draft.comment || "",
  start: props.draft.start || (isEdit ? "" : nowLabel()),
  end: props.draft.end || "",
})

// --- Список номеров заявок (как справочник клиентов) ---
const editingOrder = ref(null)
const editOrderName = ref("")
const orderEditInputs = {}
const pendingClearOrders = ref(false)

function addOrder() {
  const val = form.orderInput.trim()
  if (!val) return
  if (!form.orders.includes(val)) form.orders.push(val)
  form.orderInput = ""
}

function removeOrder(i) {
  form.orders.splice(i, 1)
  if (editingOrder.value === i) cancelOrderEdit()
}

function startOrderEdit(i) {
  editingOrder.value = i
  editOrderName.value = form.orders[i]
  nextTick(() => orderEditInputs[i]?.focus())
}

function cancelOrderEdit() {
  editingOrder.value = null
}

function saveOrderEdit(i) {
  const val = editOrderName.value.trim()
  if (val && val !== form.orders[i] && !form.orders.some((x, j) => j !== i && x === val)) {
    form.orders[i] = val
  }
  cancelOrderEdit()
}

const baseStart = ref(form.start)
const baseEnd = ref(form.end)

const rangesLines = ref(null)
const rangesDirty = ref(false)
const showRanges = ref(false)
const rangesText = ref(
  (props.draft.ranges || [])
    .map((r) => (r.stop ? `${r.start} — ${r.stop}` : r.start))
    .join("\n")
)

const hasMultiRanges = isEdit && (props.draft.intervalsCount || 1) > 1

const title = computed(() =>
  form.mode === "new" ? "Новая задача" : `Задача ${form.taskId}`
)

function submit() {
  let datesDirty = false
  if (form.start !== baseStart.value) {
    if (isDateTimeValid(form.start || "")) datesDirty = true
    else form.start = baseStart.value
  }
  if (form.end !== baseEnd.value) {
    if (!form.end || form.end === "" || isDateTimeValid(form.end || "")) datesDirty = true
    else form.end = baseEnd.value
  }
  emit("save", {
    ...form,
    _datesDirty: datesDirty,
    _rangesDirty: rangesDirty.value,
    _rangesLines: rangesDirty.value ? rangesLines.value : undefined,
  })
}

function onClose() {
  submit()
}

const statusColor = computed(() => {
  return store.statuses.filter(s => {
    return form.customStatus === s.name
  })[0]?.color ?? null
})

function onRangesSave(lines) {
  rangesLines.value = lines
  rangesDirty.value = true
  rangesText.value = lines.join("\n")
  showRanges.value = false
  const last = lines[lines.length - 1]
  const st = last.match(/^\d{2}\.\d{2}\.\d{4} \d{2}:\d{2}/)
  if (st) {
    form.start = st[0]
    baseStart.value = st[0]
  }
  const sep = last.match(/[—–-]\s*(.*)$/)
  const en = sep ? sep[1].trim() : ""
  if (!en || isDateTimeValid(en) || /^\d{2}:\d{2}$/.test(en)) {
    form.end = en
    baseEnd.value = en
  }
}
</script>

<template>
  <BaseModal :title="title" @close="onClose">
        <v-autocomplete
          v-model="form.user"
          :items="store.users.map((u) => ({ title: u, value: u }))"
          label="Пользователь"
          density="compact"
          variant="outlined"
          prepend-inner-icon="systemIcons:iconUser"

        />
      <v-text-field
        v-model="form.orderInput"
        label="Номер заявки (заказ)"
        density="compact"
        variant="outlined"
        prepend-inner-icon="mdi-clipboard-text-outline"
        append-inner-icon="systemIcons:iconPlus"
        hint="Enter или «+» — добавить в список"
        persistent-hint
        class="mb-3"
        @keydown.enter.prevent="addOrder"
        @click:append-inner="addOrder"
      />
      <v-list
        v-if="form.orders.length"
        variant="outlined"
        density="compact"
        class="order-list mb-1"
      >
        <v-list-item density="compact" v-for="(o, i) in form.orders" :key="i" class="px-2">
          <template v-if="editingOrder === i">
            <div class="d-flex align-center ga-2">
              <v-text-field
                :ref="(el) => { if (el) orderEditInputs[i] = el; }"
                v-model="editOrderName"
                density="compact"
                class="order-item"
                variant="outlined"
                hide-details
                tabindex="-1"
                @keydown.enter="saveOrderEdit(i)"
                @keydown.esc.stop="cancelOrderEdit"
              />
              <v-btn
                  icon="systemIcons:iconCheck"
                  aria-label="Сохранить"
                  variant="text"
                  size="x-small"
                  @click="saveOrderEdit(i)">
              </v-btn>
              <v-btn
                  icon="systemIcons:iconClose"
                  aria-label="Отмена"
                  variant="text"
                  size="x-small"
                  @click="cancelOrderEdit">
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
                  @click="startOrderEdit(i)">
              </v-btn>
              <v-btn icon="systemIcons:iconTrash"
                     aria-label="Удалить"
                     variant="text"
                     size="x-small"
                     tabindex="-1"
                     color="error" @click="removeOrder(i)">
              </v-btn>
            </div>
          </template>
        </v-list-item>
      </v-list>
      <div v-if="form.orders.length" class="d-flex justify-end mb-3">
        <v-btn variant="tonal" color="error" tabindex="-1" size="small" @click="pendingClearOrders = true">
          Удалить все
        </v-btn>
      </div>
      <div class="d-flex ga-4 mb-3">
        <v-autocomplete
          v-model="form.tags"
          :items="store.tags.map((t) => ({ title: t.name, value: t.name }))"
          label="Теги"
          density="compact"
          variant="outlined"
          clearable
          multiple
          chips
          prepend-inner-icon="mdi-tag-outline"
          class="flex-grow-1"
        />
        <v-combobox
          v-model="form.client"
          :items="store.clients.map((c) => c.name)"
          label="Клиент"
          density="compact"
          variant="outlined"
          clearable
          prepend-inner-icon="systemIcons:iconClient"
          hint="Можно ввести нового клиента"
          persistent-hint
          class="flex-grow-1"
        />
      </div>
      <div class="d-flex ga-4">
        <v-sheet
            class="rounded-circle"
            width="40px"
            height="40px"
            border
          :color="statusColor"
        >
        </v-sheet>
      <v-autocomplete
        v-model="form.customStatus"
        :items="store.statuses.map((s) => ({ title: s.name, value: s.name }))"
        label="Статус"
        density="compact"
        variant="outlined"
        clearable
        prepend-inner-icon="mdi-flag-outline"
        class="mb-3"
      />
        <div>
        <v-checkbox
          v-model="form.ourCar"
          label="Наша машина"
          density="compact"
          hide-details

          color="primary"
        />
        </div>
      </div>
      <div class="d-flex ga-4 mb-3">
        <v-text-field
          v-model="form.start"
          label="Начало"
          width="50%"
          placeholder="дд.мм.гггг мм:чч"
          v-maska="'##.##.#### ##:##'"
          :rules="[dateTimeRule]"
          density="compact"
          variant="outlined"
          prepend-inner-icon="mdi-clock-start"
          class="flex-grow-1"
        />
        <v-text-field
          v-model="form.end"
          label="Завершение"
          width="50%"
          placeholder="дд.мм.гггг мм:чч"
          v-maska="'##.##.#### ##:##'"
          :rules="[dateTimeRule]"
          density="compact"
          variant="outlined"
          clearable
          prepend-inner-icon="mdi-clock-end"
          class="flex-grow-1"
        />
      </div>
      <v-btn
        v-if="hasMultiRanges"
        variant="tonal"
        size="small"
        class="mb-3"
        prepend-icon="systemIcons:iconClock"
        @click="showRanges = true"
      >
        Редактировать временные диапазоны
      </v-btn>
      <v-textarea
        v-model="form.comment"
        label="Комментарий"
        rows="3"
        density="compact"
        variant="outlined"
        prepend-inner-icon="mdi-comment-outline"
      />

    <template #actions>
      <v-btn variant="text" @click="onClose">Отмена</v-btn>
      <v-btn color="primary" variant="flat" @click="submit">Сохранить</v-btn>
    </template>
  </BaseModal>

  <RangesModal
    v-if="showRanges"
    :text="rangesText"
    @save="onRangesSave"
    @close="showRanges = false"
  />

  <ConfirmDialog
    v-if="pendingClearOrders"
    title="Удалить все номера заявок?"
    message="Все номера заявок будут удалены из задачи. Продолжить?"
    @confirm="form.orders = []; pendingClearOrders = false"
    @close="pendingClearOrders = false"
  />
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