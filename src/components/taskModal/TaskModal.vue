<script setup>
import { reactive, computed, ref } from "vue"
import { useAppStore } from "../../store.js"
import RangesModal from "../range/RangesModal.vue"
import BaseModal from "../base/BaseModal.vue"
import OrderNumbersEditor from "./OrderNumbersEditor.vue"
import TaskStatusSelector from "./TaskStatusSelector.vue"
import TaskDateFields from "./TaskDateFields.vue"
import { isDateTimeValid } from "../../dateRules.js"

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
  tags: props.draft.tags || [],
  customStatus: props.draft.customStatus || "",
  ourCar: props.draft.ourCar || false,
  client: props.draft.client || "",
  comment: props.draft.comment || "",
  start: props.draft.start || (isEdit ? "" : nowLabel()),
  end: props.draft.end || "",
})

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
      class="mb-3"
    />
    <order-numbers-editor v-model:orders="form.orders" />
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
    <task-status-selector v-model:status="form.customStatus" v-model:our-car="form.ourCar" />
    <task-date-fields v-model:start="form.start" v-model:end="form.end" />
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
</template>
