<script setup>
import { ref, watch, computed } from "vue";
import { api } from "../api";
import { useAppStore } from "../store";

const store = useAppStore();
const emit = defineEmits(["close"]);

function firstOfMonth() {
  const d = new Date();
  return `${String(d.getDate()).padStart(2, "0")}.${String(d.getMonth() + 1).padStart(2, "0")}.${d.getFullYear()}`;
}

const dateFrom = ref(store.filter.dateFrom || firstOfMonth());
const dateTo = ref(store.filter.dateTo || "");

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
    tags: store.filter.tags,
    client: store.filter.client,
    user: store.filter.user,
    search: store.filter.search,
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
    const path = await api.createStats(format.value, toFilter(), dateFrom.value, dateTo.value);
    saved.value = path;
  } catch (e) {
    error.value = String(e);
  }
}

const open = computed({
  get: () => true,
  set: () => emit("close"),
});

const sections = computed(() => {
  if (!stats.value) return [];
  return [
    { title: "По пользователям", items: stats.value.byUser },
    { title: "По клиентам", items: stats.value.byClient },
    { title: "По тегам", items: stats.value.byTag },
    { title: "По месяцам", items: stats.value.byMonth },
  ];
});

watch([dateFrom, dateTo], () => loadStats());
loadStats();
</script>

<template>
  <v-dialog v-model="open" max-width="820" max-height="85vh">
    <v-card class="modal-card">
      <v-toolbar density="compact" color="primary">
        <v-toolbar-title>Статистика</v-toolbar-title>
      </v-toolbar>

      <v-card-text class="pt-5">
        <div class="d-flex ga-4 mb-3 flex-wrap">
          <v-text-field
            v-model="dateFrom"
            label="Дата с (дд.мм.гггг)"
            mask="##.##.####"
            return-masked-value
            density="compact"
            variant="outlined"
            hide-details
            style="max-width: 220px"
          />
          <v-text-field
            v-model="dateTo"
            label="Дата по (дд.мм.гггг)"
            mask="##.##.####"
            return-masked-value
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

        <template v-if="stats">
          <div class="d-flex gap-4 mb-3 flex-wrap">
            <v-chip size="small" variant="tonal" class="mr-2">
              Записей: <b class="ml-1">{{ stats.count }}</b>
            </v-chip>
            <v-chip size="small" variant="tonal" class="mr-2">
              Общее время: <b class="mono ml-1">{{ stats.totalLabel }}</b>
            </v-chip>
            <v-chip size="small" variant="tonal" class="mr-2">
              Среднее: <b class="mono ml-1">{{ stats.avgLabel }}</b>
            </v-chip>
          </div>

          <div v-for="sec in sections" :key="sec.title" class="mb-4">
            <h3 class="text-subtitle-2 mb-1">{{ sec.title }}</h3>
            <v-table v-if="sec.items.length" density="compact">
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
                  <td class="mono">{{ it.totalLabel }}</td>
                  <td class="mono">{{ it.avgLabel }}</td>
                </tr>
              </tbody>
            </v-table>
            <div v-else class="text-medium-emphasis text-caption">Нет данных</div>
          </div>
        </template>
        <div v-if="loading" class="text-medium-emphasis text-caption">Загрузка…</div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="emit('close')">Закрыть</v-btn>
        <v-btn color="primary" variant="flat" :disabled="loading" prepend-icon="systemIcons:iconExport" @click="saveStats">
          Экспорт ({{ format.toUpperCase() }})
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>