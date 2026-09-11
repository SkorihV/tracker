<script setup>
import { reactive, computed, ref } from "vue";
import { useAppStore } from "../store";
import RangesModal from "./RangesModal.vue";

const store = useAppStore();

const props = defineProps({
  draft: { type: Object, required: true },
});

const emit = defineEmits(["save", "close"]);

const DT_RE = /^\d{2}\.\d{2}\.\d{4} \d{2}:\d{2}$/;

function nowLabel() {
  const d = new Date();
  const p = (n) => String(n).padStart(2, "0");
  return `${p(d.getDate())}.${p(d.getMonth() + 1)}.${d.getFullYear()} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

const isEdit = props.draft.mode === "edit";

const form = reactive({
  mode: props.draft.mode,
  taskId: props.draft.taskId || "",
  user: props.draft.user || store.settings.username || "",
  order: props.draft.order || "",
  tags: props.draft.tags || [],
  customStatus: props.draft.customStatus || "",
  client: props.draft.client || "",
  comment: props.draft.comment || "",
  start: props.draft.start || (isEdit ? "" : nowLabel()),
  end: props.draft.end || "",
});

const baseStart = ref(form.start);
const baseEnd = ref(form.end);

const rangesLines = ref(null);
const rangesDirty = ref(false);
const showRanges = ref(false);
const rangesText = ref(
  (props.draft.ranges || [])
    .map((r) => (r.stop ? `${r.start} — ${r.stop}` : r.start))
    .join("\n")
);

const hasMultiRanges = isEdit && (props.draft.intervalsCount || 1) > 1;

const title = computed(() =>
  form.mode === "new" ? "Новая задача" : `Задача ${form.taskId}`
);

const open = computed({
  get: () => true,
  set: () => onClose(),
});

function submit() {
  let datesDirty = false;
  if (form.start !== baseStart.value) {
    if (DT_RE.test(form.start || "")) datesDirty = true;
    else form.start = baseStart.value;
  }
  if (form.end !== baseEnd.value) {
    if (!form.end || form.end === "" || DT_RE.test(form.end)) datesDirty = true;
    else form.end = baseEnd.value;
  }
  emit("save", {
    ...form,
    _datesDirty: datesDirty,
    _rangesDirty: rangesDirty.value,
    _rangesLines: rangesDirty.value ? rangesLines.value : undefined,
  });
}

function onClose() {
  submit();
}

function onRangesSave(lines) {
  rangesLines.value = lines;
  rangesDirty.value = true;
  rangesText.value = lines.join("\n");
  showRanges.value = false;
  const last = lines[lines.length - 1];
  const st = last.match(/^\d{2}\.\d{2}\.\d{4} \d{2}:\d{2}/);
  if (st) {
    form.start = st[0];
    baseStart.value = st[0];
  }
  const sep = last.match(/[—–-]\s*(.*)$/);
  const en = sep ? sep[1].trim() : "";
  if (!en || DT_RE.test(en) || /^\d{2}:\d{2}$/.test(en)) {
    form.end = en;
    baseEnd.value = en;
  }
}
</script>

<template>
  <v-dialog v-model="open" max-width="600" max-height="85vh">
    <v-card class="modal-card">
      <v-toolbar density="compact" color="primary">
        <v-toolbar-title>{{ title }}</v-toolbar-title>
      </v-toolbar>
      <v-card-text class="pt-5">
        <div v-if="isEdit" class="d-flex ga-4 mb-3">
          <v-text-field
            :model-value="form.taskId"
            label="ID"
            density="compact"
            variant="outlined"
            readonly
            class="mono"
            style="max-width: 220px"
          />
          <v-autocomplete
            v-model="form.user"
            :items="store.users.map((u) => ({ title: u, value: u }))"
            label="Пользователь"
            density="compact"
            variant="outlined"
            prepend-inner-icon="systemIcons:iconUser"
            class="flex-grow-1"
          />
        </div>
        <v-autocomplete
          v-else
          v-model="form.user"
          :items="store.users.map((u) => ({ title: u, value: u }))"
          label="Пользователь"
          density="compact"
          variant="outlined"
          prepend-inner-icon="systemIcons:iconUser"
          class="mb-3"
        />
        <v-text-field
          v-model="form.order"
          label="Номер заявки (заказ)"
          placeholder="например 12345"
          density="compact"
          variant="outlined"
          class="mb-3"
        />
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
            class="flex-grow-1"
          />
          <v-autocomplete
            v-model="form.client"
            :items="store.clients.map((c) => ({ title: c.name, value: c.name }))"
            label="Клиент"
            density="compact"
            variant="outlined"
            clearable
            class="flex-grow-1"
          />
        </div>
        <v-autocomplete
          v-model="form.customStatus"
          :items="store.statuses.map((s) => ({ title: s.name, value: s.name }))"
          label="Статус"
          density="compact"
          variant="outlined"
          clearable
          class="mb-3"
        />
        <div class="d-flex ga-4 mb-3">
          <v-text-field
            v-model="form.start"
            label="Начало"
            placeholder="дд.мм.гггг чч:мм"
            density="compact"
            variant="outlined"
            mask="##.##.#### ##:##"
            return-masked-value
            class="flex-grow-1"
          />
          <v-text-field
            v-model="form.end"
            label="Завершение"
            placeholder="дд.мм.гггг чч:мм"
            density="compact"
            variant="outlined"
            mask="##.##.#### ##:##"
            return-masked-value
            clearable
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
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="onClose">Отмена</v-btn>
        <v-btn color="primary" variant="flat" @click="submit">Сохранить</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <RangesModal
    v-if="showRanges"
    :text="rangesText"
    @save="onRangesSave"
    @close="showRanges = false"
  />
</template>