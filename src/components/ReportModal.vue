<script setup>
import { ref, watch, computed } from "vue";
import { api } from "../api";
import { useAppStore } from "../store";
import { dateRule } from "../dateRules.js";

const store = useAppStore();
const emit = defineEmits(["close"]);

const formats = ["txt", "md", "csv", "xlsx"];
const format = ref("txt");
const dateFrom = ref(store.filter.dateFrom || "");
const dateTo = ref(store.filter.dateTo || "");

const preview = ref(null);
const loading = ref(false);
const error = ref("");
const saved = ref("");

function toFilter() {
  return {
    dateFrom: dateFrom.value,
    dateTo: dateTo.value,
    tags: store.filter.tags,
    client: store.filter.client,
    user: store.filter.user,
    search: store.filter.search,
  };
}

async function loadPreview() {
  loading.value = true;
  error.value = "";
  try {
    preview.value = await api.reportPreview(toFilter());
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
    const path = await api.createReport(format.value, toFilter(), dateFrom.value, dateTo.value);
    saved.value = path;
  } catch (e) {
    error.value = String(e);
  }
}

const open = computed({
  get: () => true,
  set: () => emit("close"),
});

watch([dateFrom, dateTo, () => store.filter.search], () => loadPreview());
loadPreview();
</script>

<template>
  <v-dialog v-model="open" max-width="1000" max-height="85vh">
    <v-card class="modal-card">
      <v-toolbar v-dialog-drag density="compact" color="primary">
        <v-toolbar-title>Отчёт</v-toolbar-title>
        <v-spacer />
        <v-btn icon variant="text" title="Закрыть" @click="emit('close')">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-toolbar>

      <v-card-text class="pt-5">
        <div class="d-flex ga-4 mb-3 flex-wrap">
          <v-text-field
            v-model="dateFrom"
            label="Дата с (дд.мм.гггг)"
            placeholder="пусто = без ограничения"
            v-maska="'##.##.####'"
            :rules="[dateRule]"
            density="compact"
            variant="outlined"
            hide-details
            style="max-width: 220px"
          />
          <v-text-field
            v-model="dateTo"
            label="Дата по (дд.мм.гггг)"
            placeholder="пусто = без ограничения"
            v-maska="'##.##.####'"
            :rules="[dateRule]"
            density="compact"
            variant="outlined"
            hide-details
            style="max-width: 220px"
          />
          <v-autocomplete
            v-model="format"
            :items="formats.map((f) => ({ title: f.toUpperCase(), value: f }))"
            label="Формат файла"
            density="compact"
            variant="outlined"
            hide-details
            style="max-width: 180px"
          />
        </div>

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
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="emit('close')">Закрыть</v-btn>
        <v-btn color="primary" variant="flat" :disabled="loading" prepend-icon="systemIcons:iconExport" @click="saveReport">
          Сохранить в reports/
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>