<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from "vue";
import { api } from "../api";
import { state as appState } from "../store";

const emit = defineEmits(["close"]);

const formats = ["txt", "md", "csv", "xlsx"];
const format = ref("txt");
const dateFrom = ref(appState.filter.dateFrom || "");
const dateTo = ref(appState.filter.dateTo || "");

const preview = ref(null);
const loading = ref(false);
const error = ref("");
const saved = ref("");

function toFilter() {
  return {
    dateFrom: dateFrom.value,
    dateTo: dateTo.value,
    tag: appState.filter.tag,
    client: appState.filter.client,
    user: appState.filter.user,
    search: appState.filter.search,
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
    const path = await api.createReport(
      format.value,
      toFilter(),
      dateFrom.value,
      dateTo.value
    );
    saved.value = path;
  } catch (e) {
    error.value = String(e);
  }
}

function onKeydown(e) {
  if (e.key === "Escape") emit("close");
}

watch([dateFrom, dateTo, () => appState.filter.search], () => loadPreview());

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  loadPreview();
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal modal-wide">
      <div class="modal-head">
        <span>📄 Отчёт</span>
        <button class="icon" @click="emit('close')">✕</button>
      </div>
      <div class="modal-body">
        <div class="form-grid2">
          <div class="form-row">
            <label>Дата с (дд.мм.гггг)</label>
            <input v-model="dateFrom" placeholder="пусто = без ограничения" />
          </div>
          <div class="form-row">
            <label>Дата по (дд.мм.гггг)</label>
            <input v-model="dateTo" placeholder="пусто = без ограничения" />
          </div>
        </div>
        <div class="form-row">
          <label>Формат файла</label>
          <select v-model="format">
            <option v-for="f in formats" :key="f" :value="f">{{ f.toUpperCase() }}</option>
          </select>
        </div>

        <div v-if="error" class="banner-error">{{ error }}</div>
        <div v-if="saved" class="banner-ok">Сохранено: {{ saved }}</div>

        <template v-if="preview">
          <div class="stats-summary">
            <span>Записей: <b>{{ preview.count }}</b></span>
            <span>Общее время: <b>{{ preview.totalLabel }}</b></span>
            <span v-for="t in preview.byTag" :key="t.name">{{ t.name }}: {{ t.timeLabel }}</span>
          </div>
          <table class="grid compact">
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
                <td>{{ r.start }}</td>
                <td>{{ r.end }}</td>
                <td>{{ r.user }}</td>
                <td>{{ r.order }}</td>
                <td>{{ r.client }}</td>
                <td>{{ r.tag }}</td>
                <td>{{ r.elapsedLabel }}</td>
                <td class="comment-cell">{{ r.comment }}</td>
              </tr>
            </tbody>
          </table>
        </template>
        <div v-if="loading" class="muted">Загрузка…</div>
      </div>
      <div class="modal-foot">
        <button @click="emit('close')">Закрыть</button>
        <button class="primary" :disabled="loading" @click="saveReport">
          💾 Сохранить в reports/
        </button>
      </div>
    </div>
  </div>
</template>