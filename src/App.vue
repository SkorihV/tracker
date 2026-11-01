<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue"
import { useAppStore } from "./stores/app.js"
import { useSettingsStore } from "./stores/settings.js"
import { useTasksStore } from "./stores/tasks.js"
import FilterBar from "./components/filter/FilterBar.vue"
import TaskTable from "./components/task/TaskTable.vue"
import TaskModal from "./components/taskModal/TaskModal.vue"
import SettingsModal from "./components/settings/SettingsModal.vue"
import ReportModal from "./components/report/ReportModal.vue"
import StatsModal from "./components/stats/StatsModal.vue"
import ImportExportModal from "./components/import/ImportExportModal.vue"
import BaseConfirmDeleteModal from "./components/base/BaseConfirmDeleteModal.vue"
import HeaderLayout from "./components/header/HeaderLayout.vue"

const appStore = useAppStore()
const settingsStore = useSettingsStore()
const tasksStore = useTasksStore()
const editing = ref(null)
const showReport = ref(false)
const showStats = ref(false)
const showImportExport = ref(false)
const confirmDelete = ref(null)
const searchRef = ref(null)
const selectedIds = ref(new Set())

function focusSearch() {
  searchRef.value?.focus()
  const el = searchRef.value?.$el?.querySelector?.("input")
  if (el) el.select()
}

async function newTask() {
  if (!settingsStore.settings.username) {
    appStore.showSettings = true
    return
  }
  editing.value = {
    mode: "new",
    user: settingsStore.settings.username,
    orders: [],
    tags: [],
    client: "",
    comment: "",
    customStatus: "",
    ourCar: false,
  }
}

function editTask(task) {
  editing.value = {
    mode: "edit",
    taskId: task.taskId,
    user: task.user,
    orders: task.orders || [],
    tags: task.tags || [],
    client: task.client,
    comment: task.comment,
    customStatus: task.status || "",
    ourCar: task.ourCar || false,
    // В окне редактирования — даты текущего (последнего) диапазона; в таблице — первое начало и последнее завершение.
    start: task.currentStart || task.start || "",
    end: task.currentEnd || "",
    intervalsCount: task.intervalsCount || 1,
    ranges: task.ranges || [],
  }
}

async function onSaveTask(draft) {
  const fields = {
    user: draft.user,
    orders: draft.orders || [],
    tags: draft.tags || [],
    client: draft.client,
    comment: draft.comment,
    customStatus: draft.customStatus || "",
    ourCar: draft.ourCar || false,
  }
  let id = draft.taskId
  if (draft.mode === "new") {
    const created = await tasksStore.addTask(fields)
    id = created.taskId
  } else {
    await tasksStore.updateTask(id, fields)
  }
  if (draft._datesDirty) {
    await tasksStore.updateTaskDates(id, draft.start || "", draft.end || "")
  }
  if (draft._rangesDirty && draft._rangesLines?.length) {
    await tasksStore.updateTaskIntervals(id, draft._rangesLines)
  }
  editing.value = null
}

async function onDeleteTasks(ids) {
  if (!ids.length) return
  confirmDelete.value = [...ids]
}

async function onConfirmDelete() {
  const ids = confirmDelete.value
  confirmDelete.value = null
  if (ids?.length) await tasksStore.removeTasks(ids)
}

function onKeydown(e) {
  const tag = (e.target.tagName || "").toLowerCase()
  const typing = tag === "input" || tag === "textarea" || tag === "select"
  if (typing) {
    if (e.key === "Escape" && tag === "input" && e.target === searchRef.value?.$el?.querySelector?.("input")) {
      tasksStore.filter.search = ""
      appStore.refreshQuery()
    }
    return
  }
  if ((e.ctrlKey || e.metaKey) && (e.key === "n" || e.key === "N")) {
    e.preventDefault()
    newTask()
  } else if ((e.ctrlKey || e.metaKey) && (e.key === "f" || e.key === "F")) {
    e.preventDefault()
    focusSearch()
  } else if (e.key === "Delete" || e.key === "Backspace") {
    const ids = Array.from(selectedIds.value)
    if (ids.length) onDeleteTasks(ids)
  }
}

// Таблицу нужно обновлять раз в секунду, только пока где-то идёт таймер.
let timer = null
onMounted(async () => {
  window.addEventListener("keydown", onKeydown)
  await appStore.init()
  timer = window.setInterval(() => {
    if (tasksStore.hasRunning) appStore.refreshQuery()
  }, 1000)
})

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown)
  if (timer) window.clearInterval(timer)
})
</script>

<template>
  <v-app>
    <header-layout @new-task="newTask" />

    <v-main class="px-4 pt-4 pb-1 d-flex flex-column">
      <v-alert
        v-if="appStore.error"
        max-height="100px"
        type="error"
        density="compact"
        class="mb-3 flex-shrink-0"
        closable
        @click:close="appStore.error = ''"
      >
        {{ appStore.error }}
      </v-alert>
      <FilterBar class="flex-shrink-0" />

      <TaskTable v-model:selected-ids="selectedIds" class="flex-grow-1" @edit="editTask" @delete="onDeleteTasks" />

      <div class="d-flex align-center justify-space-between flex-wrap ga-4 pt-1 flex-shrink-0">
        <div class="d-flex align-center ga-3">
          <span class="text-body-2 text-medium-emphasis">
            Задачи: <b>{{ tasksStore.totals.count }}</b>
          </span>
          <div class="d-flex flex-column">
            <span
              class="text-body-2 text-medium-emphasis"
              title="Время каждой задачи складывается отдельно: параллельные задачи считаются несколько раз"
            >
              Сумма по задачам: <b class="mono">{{ tasksStore.totals.timeLabel }}</b>
            </span>
            <span
              class="text-body-2 text-medium-emphasis"
              title="Время, в которое шла хотя бы одна задача: пересечения считаются один раз"
            >
              Фактическое время: <b class="mono">{{ tasksStore.totals.actualLabel }}</b>
            </span>
          </div>
          <v-btn variant="tonal" size="small" prepend-icon="systemIcons:iconClock" @click="showReport = true"
            >Отчёт</v-btn
          >
          <v-btn variant="tonal" size="small" prepend-icon="systemIcons:iconChart" @click="showStats = true"
            >Статистика</v-btn
          >
          <v-btn variant="tonal" size="small" prepend-icon="systemIcons:iconExport" @click="showImportExport = true"
            >Экспорт/Импорт</v-btn
          >
        </div>
        <v-btn
          v-if="selectedIds.size"
          color="error"
          variant="tonal"
          prepend-icon="systemIcons:iconTrash"
          @click="onDeleteTasks(Array.from(selectedIds))"
        >
          Удалить выбранное ({{ selectedIds.size }})
        </v-btn>
      </div>
    </v-main>

    <TaskModal v-if="editing" :draft="editing" @save="onSaveTask" @close="editing = null" />
    <SettingsModal v-if="appStore.showSettings" @close="appStore.showSettings = false" />
    <ReportModal v-if="showReport" @close="showReport = false" />
    <StatsModal v-if="showStats" @close="showStats = false" />
    <ImportExportModal v-if="showImportExport" @close="showImportExport = false" />
    <BaseConfirmDeleteModal
      v-if="confirmDelete"
      :ids="confirmDelete"
      @confirm="onConfirmDelete"
      @close="confirmDelete = null"
    />
  </v-app>
</template>
