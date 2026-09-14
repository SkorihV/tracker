<script setup>
import { ref, computed, watch } from "vue"
import { save } from "@tauri-apps/plugin-dialog"
import { api } from "../api"
import { useAppStore } from "../store"
import {reportFilter, today} from "../reportFilter"
import BaseModal from "./BaseModal.vue"
import ReportPeriodFields from "./ReportPeriodFields.vue"

const store = useAppStore()
const emit = defineEmits(["close"])

// Колонки предпросмотра — как в таблице Задач (без служебной «Действия»).
const PREVIEW_COLUMNS = {
  time: { title: "Время", cell: (r) => r.elapsedLabel, mono: true },
  status: { title: "Статус", cell: (r) => r.customStatus },
  ourCar: { title: "Наша машина", cell: (r) => (r.ourCar ? "Да" : "") },
  mode: { title: "Режим", cell: (r) => r.mode },
  order: { title: "Заявка", cell: (r) => r.order },
  client: { title: "Клиент", cell: (r) => r.client },
  tags: { title: "Тег", cell: (r) => (r.tags || []).join(", ") },
  start: { title: "Начало", cell: (r) => r.start, mono: true },
  end: { title: "Завершение", cell: (r) => r.end, mono: true },
  user: { title: "Пользователь", cell: (r) => r.user },
  comment: { title: "Комментарий", cell: (r) => r.comment },
  taskId: { title: "ID", cell: (r) => r.taskId },
}

const previewColumns = computed(() =>
  store.visibleColumns
    .filter((c) => c.key !== "actions" && PREVIEW_COLUMNS[c.key])
    .map((c) => PREVIEW_COLUMNS[c.key])
)

const format = ref("xlsx")
const dateFrom = ref(store.filter.dateFrom || today())
const dateTo = ref(store.filter.dateTo || today())

const preview = ref(null)
const loading = ref(false)
const error = ref("")
const saved = ref("")

function tsNow() {
  const d = new Date()
  const p = (n) => String(n).padStart(2, "0")
  return `${d.getFullYear()}${p(d.getMonth()+1)}${p(d.getDate())}_${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`
}

async function loadPreview() {
  loading.value = true
  error.value = ""
  try {
    preview.value = await api.reportPreview(reportFilter(dateFrom.value, dateTo.value, store.filter))
  } catch (e) {
    error.value = String(e)
    preview.value = null
  } finally {
    loading.value = false
  }
}

async function saveReport() {
  error.value = ""
  saved.value = ""
  try {
    const ext = format.value
    const target = await save({
      defaultPath: `report_${tsNow()}.${ext}`,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    })
    if (!target) return
    const path = await api.createReport(ext, reportFilter(dateFrom.value, dateTo.value, store.filter), dateFrom.value, dateTo.value, target)
    saved.value = path
  } catch (e) {
    error.value = String(e)
  }
}

watch([dateFrom, dateTo, () => store.filter.search], () => loadPreview())
loadPreview()
</script>

<template>
  <BaseModal title="Отчёт" max-width="1000" width="1000" max-height="85vh" @close="emit('close')">
    <ReportPeriodFields v-model:date-from="dateFrom" v-model:date-to="dateTo" v-model:format="format" empty-hint="пусто = без ограничения" />

    <v-alert v-if="error" type="error" density="compact" variant="tonal" class="mb-3">
      {{ error }}
    </v-alert>
    <v-alert v-if="saved" type="success" density="compact" variant="tonal" class="mb-3">
      Сохранено: {{ saved }}
    </v-alert>

    <template v-if="preview">
      <div class="d-flex gap-4 mb-3 flex-wrap">
        <v-chip size="small" variant="tonal" class="mr-2">
          Записей: <b class="ml-1">{{ preview.count }}</b>
        </v-chip>
        <v-chip size="small" variant="tonal" class="mr-2">
          Общее время: <b class="mono ml-1">{{ preview.totalLabel }}</b>
        </v-chip>
        <v-chip v-for="t in preview.byTag" :key="t.name" size="small" variant="tonal" class="mr-2">
          {{ t.name }}: {{ t.timeLabel }}
        </v-chip>
      </div>

      <v-table density="compact" class="max-width-table">
        <thead>
          <tr>
            <th v-for="col in previewColumns" :key="col.title">{{ col.title }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="r in preview.rows" :key="r.taskId">
            <td v-for="col in previewColumns" :key="col.title" :class="{ mono: col.mono }">{{ col.cell(r) }}</td>
          </tr>
        </tbody>
      </v-table>
    </template>
    <div v-if="loading" class="text-medium-emphasis text-caption">Загрузка</div>

    <template #actions>
      <v-btn variant="text" @click="emit('close')">Закрыть</v-btn>
      <v-btn color="primary" variant="flat" :disabled="loading" prepend-icon="systemIcons:iconExport" @click="saveReport">
        Сохранить ({{ format.toUpperCase() }})
      </v-btn>
    </template>
  </BaseModal>
</template>