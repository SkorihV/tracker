<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue"
import { useAppStore } from "./store"
import FilterBar from "./components/filter/FilterBar.vue"
import TaskTable from "./components/task/TaskTable.vue"
import TaskModal from "./components/taskModal/TaskModal.vue"
import SettingsModal from "./components/settings/SettingsModal.vue"
import ReportModal from "./components/ReportModal.vue"
import StatsModal from "./components/StatsModal.vue"
import ImportExportModal from "./components/ImportExportModal.vue"
import ConfirmDeleteModal from "./components/ConfirmDeleteModal.vue"
import HeaderLayout from "./components/header/HeaderLayout.vue"

const store = useAppStore()
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
  if (!store.settings.username) {
    store.showSettings = true
    return
  }
  editing.value = {
    mode: "new",
    user: store.settings.username,
    order: "",
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
    order: task.order,
    tags: task.tags || [],
    client: task.client,
    comment: task.comment,
    customStatus: task.status || "",
    ourCar: task.ourCar || false,
    start: task.start || "",
    end: task.end || "",
    intervalsCount: task.intervalsCount || 1,
    ranges: task.ranges || [],
  }
}

async function onSaveTask(draft) {
  const fields = {
    user: draft.user,
    order: draft.order,
    tags: draft.tags || [],
    client: draft.client,
    comment: draft.comment,
    customStatus: draft.customStatus || "",
    ourCar: draft.ourCar || false,
  }
  let id = draft.taskId
  if (draft.mode === "new") {
    const created = await store.addTask(fields)
    id = created.taskId
  } else {
    await store.updateTask(id, fields)
  }
  if (draft._datesDirty) {
    await store.updateTaskDates(id, draft.start || "", draft.end || "")
  }
  if (draft._rangesDirty && draft._rangesLines?.length) {
    await store.updateTaskIntervals(id, draft._rangesLines)
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
  if (ids?.length) await store.removeTasks(ids)
}


function onKeydown(e) {
  const tag = (e.target.tagName || "").toLowerCase()
  const typing = tag === "input" || tag === "textarea" || tag === "select"
  if (typing) {
    if (e.key === "Escape" && tag === "input" && e.target === searchRef.value?.$el?.querySelector?.("input")) {
      store.filter.search = ""
      store.refreshQuery()
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

let timer = null
onMounted(async () => {
  window.addEventListener("keydown", onKeydown)
  await store.init()
  await store.refreshQuery()
  timer = window.setInterval(async () => {
    await store.refreshQuery()
  }, 1000)
})

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown)
  if (timer) window.clearInterval(timer)
})
</script>

<template>
  <v-app>
     <header-layout
      @new-task="newTask"
     />

    <v-main class="pa-4 d-flex flex-column">
      <v-alert v-if="store.error" max-height="100px" type="error" density="compact" class="mb-3" closable @click:close="store.error = ''">
        {{ store.error }}
      </v-alert>

      <FilterBar />

      <TaskTable
        class="flex-grow-1"
        v-model:selected-ids="selectedIds"
        @edit="editTask"
        @delete="onDeleteTasks"
      />

      <div class="d-flex align-center justify-space-between flex-wrap ga-4 pt-3">
        <div class="d-flex align-center ga-3">
          <span class="text-body-2 text-medium-emphasis">
            Задачи: <b>{{ store.totals.count }}</b>
          </span>
          <span class="text-body-2 text-medium-emphasis">
            Общее время: <b class="mono">{{ store.totals.timeLabel }}</b>
          </span>
          <v-btn variant="tonal" size="small" prepend-icon="systemIcons:iconClock" @click="showReport = true">Отчёт</v-btn>
          <v-btn variant="tonal" size="small" prepend-icon="systemIcons:iconChart" @click="showStats = true">Статистика</v-btn>
          <v-btn variant="tonal" size="small" prepend-icon="systemIcons:iconExport" @click="showImportExport = true">Экспорт/Импорт</v-btn>
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
    <SettingsModal v-if="store.showSettings" @close="store.showSettings = false" />
    <ReportModal v-if="showReport" @close="showReport = false" />
    <StatsModal v-if="showStats" @close="showStats = false" />
    <ImportExportModal v-if="showImportExport" @close="showImportExport = false" />
    <ConfirmDeleteModal
      v-if="confirmDelete"
      :ids="confirmDelete"
      @confirm="onConfirmDelete"
      @close="confirmDelete = null"
    />
  </v-app>
</template>