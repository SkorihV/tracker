<script setup>
import { ref, watch, computed } from "vue"
import { save } from "@tauri-apps/plugin-dialog"
import { api } from "../../api.js"
import { useAppStore } from "../../store.js"
import { reportFilter, today } from "../../reportFilter.js"
import BaseModal from "../base/BaseModal.vue"
import BasePeriodFields from "../base/BasePeriodFields.vue"

const store = useAppStore()
const emit = defineEmits(["close"])

const dateFrom = ref(store.filter.dateFrom || today())
const dateTo = ref(store.filter.dateTo || today())

const format = ref("xlsx")

const stats = ref(null)
const loading = ref(false)
const error = ref("")
const saved = ref("")

function tsNow() {
  const d = new Date()
  const p = (n) => String(n).padStart(2, "0")
  return `${d.getFullYear()}${p(d.getMonth()+1)}${p(d.getDate())}_${p(d.getHours())}${p(d.getMinutes())}${p(d.getSeconds())}`
}

async function loadStats() {
  loading.value = true
  error.value = ""
  try {
    stats.value = await api.statsPreview(reportFilter(dateFrom.value, dateTo.value, store.filter))
  } catch (e) {
    error.value = String(e)
    stats.value = null
  } finally {
    loading.value = false
  }
}

async function saveStats() {
  error.value = ""
  saved.value = ""
  try {
    const ext = format.value
    const target = await save({
      defaultPath: `stats_${tsNow()}.${ext}`,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    })
    if (!target) return
    const path = await api.createStats(ext, reportFilter(dateFrom.value, dateTo.value, store.filter), dateFrom.value, dateTo.value, target)
    saved.value = path
  } catch (e) {
    error.value = String(e)
  }
}

const sections = computed(() => {
  if (!stats.value) return []
  return [
    { title: "По пользователям", items: stats.value.byUser },
    { title: "По клиентам", items: stats.value.byClient },
    { title: "По тегам", items: stats.value.byTag },
    { title: "По месяцам", items: stats.value.byMonth },
  ]
})

watch([dateFrom, dateTo], () => loadStats())
loadStats()
</script>

<template>
  <BaseModal title="Статистика" max-width="1000" width="1000" max-height="85vh" @close="emit('close')">
    <BasePeriodFields v-model:date-from="dateFrom" v-model:date-to="dateTo" v-model:format="format" />

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
        <v-table v-if="sec.items.length" density="compact" class="max-width-table">
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

    <template #actions>
      <v-btn variant="text" @click="emit('close')">Закрыть</v-btn>
      <v-btn color="primary" variant="flat" :disabled="loading" prepend-icon="systemIcons:iconExport" @click="saveStats">
        Экспорт ({{ format.toUpperCase() }})
      </v-btn>
    </template>
  </BaseModal>
</template>