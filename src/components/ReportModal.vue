<script setup>
import { ref, watch } from "vue";
import { save } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { useAppStore } from "../store";
import { reportFilter } from "../reportFilter";
import BaseModal from "./BaseModal.vue";
import ReportPeriodFields from "./ReportPeriodFields.vue";

const store = useAppStore();
const emit = defineEmits(["close"]);

const format = ref("txt");
const dateFrom = ref(store.filter.dateFrom || "");
const dateTo = ref(store.filter.dateTo || "");

const preview = ref(null);
const loading = ref(false);
const error = ref("");
const saved = ref("");

function tsNow() {
  const d = new Date();
  const p = (n) => String(n).padStart(2, "0");
  return `${d.getFullYear()}${p(d.getMonth()+1)}${p(d.getDate())}_${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`;
}

async function loadPreview() {
  loading.value = true;
  error.value = "";
  try {
    preview.value = await api.reportPreview(reportFilter(dateFrom.value, dateTo.value, store.filter));
  } catch (e) {
    error.value = String(e);
    preview.value = null;
  } finally {
    loading.value = false;
  }
}

async function saveReport() {
  error.value = "";
  saved.value = "";
  try {
    const ext = format.value;
    const target = await save({
      defaultPath: `report_${tsNow()}.${ext}`,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    });
    if (!target) return;
    const path = await api.createReport(ext, reportFilter(dateFrom.value, dateTo.value, store.filter), dateFrom.value, dateTo.value, target);
    saved.value = path;
  } catch (e) {
    error.value = String(e);
  }
}

watch([dateFrom, dateTo, () => store.filter.search], () => loadPreview());
loadPreview();
</script>

<template>
  <BaseModal title="Отчёт" max-width="1000" max-height="85vh" @close="emit('close')">
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
            <th>Начало</th>
            <th>Конец</th>
            <th>Пользователь</th>
            <th>Заявка</th>
            <th>Клиент</th>
            <th>Тег</th>
            <th>Время</th>
            <th>Комментарий</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="r in preview.rows" :key="r.taskId">
            <td class="mono">{{ r.start }}</td>
            <td class="mono">{{ r.end }}</td>
            <td>{{ r.user }}</td>
            <td>{{ r.order }}</td>
            <td>{{ r.client }}</td>
            <td>{{ (r.tags || []).join(", ") }}</td>
            <td class="mono">{{ r.elapsedLabel }}</td>
            <td class="comment-cell">{{ r.comment }}</td>
          </tr>
        </tbody>
      </v-table>
    </template>
    <div v-if="loading" class="text-medium-emphasis text-caption">Загрузка…</div>

    <template #actions>
      <v-btn variant="text" @click="emit('close')">Закрыть</v-btn>
      <v-btn color="primary" variant="flat" :disabled="loading" prepend-icon="systemIcons:iconExport" @click="saveReport">
        Сохранить…
      </v-btn>
    </template>
  </BaseModal>
</template>