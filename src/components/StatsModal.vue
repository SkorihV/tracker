<script setup>
import { ref, onMounted, onBeforeUnmount, watch, computed } from "vue";
import { api } from "../api";
import { state as appState } from "../store";

const emit = defineEmits(["close"]);

function firstOfMonth() {
  const d = new Date();
  return `${String(d.getDate()).padStart(2, "0")}.${String(d.getMonth() + 1).padStart(2, "0")}.${d.getFullYear()}`;
}

const dateFrom = ref(appState.filter.dateFrom || firstOfMonth());
const dateTo = ref(appState.filter.dateTo || "");

const formats = ["txt", "md", "csv", "xlsx"];
const format = ref("txt");

const stats = ref(null);
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

async function loadStats() {
  loading.value = true;
  error.value = "";
  try {
    stats.value = await api.statsPreview(toFilter());
  } catch (e) {
    error.value = String(e);
    stats.value = null;
  } finally {
    loading.value = false;
  }
}

async function saveStats() {
  error.value = "";
  saved.value = "";
  try {
    const path = await api.createStats(
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

watch([dateFrom, dateTo], () => loadStats());

const sections = computed(() => {
  if (!stats.value) return [];
  return [
    { title: "По пользователям", items: stats.value.byUser },
    { title: "По клиентам", items: stats.value.byClient },
    { title: "По тегам", items: stats.value.byTag },
    { title: "По месяцам", items: stats.value.byMonth },
  ];
});

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  loadStats();
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal modal-suit">
      <div class="modal-head">
        <span>📊 Статистика</span>
        <button class="icon" @click="emit('close')">✕</button>
      </div>
      <div class="modal-body">
        <div class="form-grid2">
          <div class="form-row">
            <label>Дата с (дд.мм.гггг)</label>
            <input v-model="dateFrom" />
          </div>
          <div class="form-row">
            <label>Дата по (дд.мм.гггг)</label>
            <input v-model="dateTo" />
          </div>
        </div>
        <div class="form-row" style="max-width: 180px">
          <label>Формат файла</label>
          <select v-model="format">
            <option v-for="f in formats" :key="f" :value="f">{{ f.toUpperCase() }}</option>
          </select>
        </div>

        <div v-if="error" class="banner-error">{{ error }}</div>
        <div v-if="saved" class="banner-ok">Сохранено: {{ saved }}</div>

        <template v-if="stats">
          <div class="stats-summary">
            <span>Записей: <b>{{ stats.count }}</b></span>
            <span>Общее время: <b>{{ stats.totalLabel }}</b></span>
            <span>Среднее: <b>{{ stats.avgLabel }}</b></span>
          </div>
          <div v-for="sec in sections" :key="sec.title" class="stats-section">
            <h3>{{ sec.title }}</h3>
            <table v-if="sec.items.length" class="grid compact">
              <thead>
                <tr>
                  <th>Имя</th>
                  <th>Задач</th>
                  <th>Общее время</th>
                  <th>Среднее</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="it in sec.items" :key="it.name">
                  <td>{{ it.name }}</td>
                  <td>{{ it.count }}</td>
                  <td>{{ it.totalLabel }}</td>
                  <td>{{ it.avgLabel }}</td>
                </tr>
              </tbody>
            </table>
            <div v-else class="muted">Нет данных</div>
          </div>
        </template>
        <div v-if="loading" class="muted">Загрузка…</div>
      </div>
      <div class="modal-foot">
        <button @click="emit('close')">Закрыть</button>
        <button class="primary" :disabled="loading" @click="saveStats">
          💾 Экспорт ({{ format.toUpperCase() }})
        </button>
      </div>
    </div>
  </div>
</template>