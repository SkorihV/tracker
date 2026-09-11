<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import { useAppStore } from "./store";
import FilterBar from "./components/filter/FilterBar.vue";
import TaskTable from "./components/task/TaskTable.vue";
import TaskFormModal from "./components/TaskFormModal.vue";
import SettingsModal from "./components/settings/SettingsModal.vue";
import ReportModal from "./components/ReportModal.vue";
import StatsModal from "./components/StatsModal.vue";
import ImportExportModal from "./components/ImportExportModal.vue";

const store = useAppStore();
const editing = ref(null);
const showSettings = ref(false);
const showReport = ref(false);
const showStats = ref(false);
const showImportExport = ref(false);
const searchRef = ref(null);
const selectedIds = ref(new Set());

function onSelectionChange(set) {
  selectedIds.value = new Set(set);
}

function focusSearch() {
  searchRef.value?.focus();
  const el = searchRef.value?.$el?.querySelector?.("input");
  if (el) el.select();
}

async function newTask() {
  if (!store.settings.username) {
    showSettings.value = true;
    return;
  }
  editing.value = {
    mode: "new",
    user: store.settings.username,
    order: "",
    tags: [],
    client: "",
    comment: "",
    customStatus: "",
  };
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
    customStatus: task.customStatus || "",
    start: task.startLabel || "",
    end: task.endLabel || "",
    intervalsCount: task.intervalsCount || 1,
    ranges: task.ranges || [],
  };
}

async function onSaveTask(draft) {
  const fields = {
    user: draft.user,
    order: draft.order,
    tags: draft.tags || [],
    client: draft.client,
    comment: draft.comment,
    customStatus: draft.customStatus || "",
  };
  let id = draft.taskId;
  if (draft.mode === "new") {
    const created = await store.addTask(fields);
    id = created.taskId;
  } else {
    await store.updateTask(id, fields);
  }
  if (draft._datesDirty) {
    await store.updateTaskDates(id, draft.start || "", draft.end || "");
  }
  if (draft._rangesDirty && draft._rangesLines?.length) {
    await store.updateTaskIntervals(id, draft._rangesLines);
  }
  editing.value = null;
}

async function onDeleteTasks(ids) {
  if (!ids.length) return;
  const msg = ids.length === 1
    ? "Удалить задачу?"
    : `Удалить выбранные задачи (${ids.length})?`;
  if (window.confirm(msg)) {
    await store.removeTasks(ids);
  }
}

function onGroupingChange(value) {
  store.applySettings(store.settings.username, value);
}

function onKeydown(e) {
  const tag = (e.target.tagName || "").toLowerCase();
  const typing = tag === "input" || tag === "textarea" || tag === "select";
  if (typing) {
    if (e.key === "Escape" && tag === "input" && e.target === searchRef.value?.$el?.querySelector?.("input")) {
      store.filter.search = "";
      store.refreshQuery();
    }
    return;
  }
  if ((e.ctrlKey || e.metaKey) && (e.key === "n" || e.key === "N")) {
    e.preventDefault();
    newTask();
  } else if ((e.ctrlKey || e.metaKey) && (e.key === "f" || e.key === "F")) {
    e.preventDefault();
    focusSearch();
  } else if (e.key === "Delete" || e.key === "Backspace") {
    const ids = Array.from(selectedIds.value);
    if (ids.length) onDeleteTasks(ids);
  }
}

let timer = null;
onMounted(async () => {
  window.addEventListener("keydown", onKeydown);
  await store.init();
  await store.refreshQuery();
  timer = window.setInterval(async () => {
    await store.refreshQuery();
  }, 1000);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  if (timer) window.clearInterval(timer);
});
</script>

<template>
  <v-app>
    <v-container fluid class="border-b py-0 px-4 d-flex justify-center align-center ga-4">
      <template #prepend>

      </template>
      <v-sheet min-width="150px"><v-btn icon="systemIcons:iconTimer" variant="text" aria-label="Time Tracker" />Time Tracker</v-sheet>



      <v-autocomplete
        :model-value="store.settings.grouping"
        :items="[
          { title: 'Нет', value: 'none' },
          { title: 'По дням', value: 'day' },
          { title: 'По клиентам', value: 'client' },
        ]"
        label="Группировка"
        variant="solo"
        density="compact"
        hide-details
        min-width="150px"
        @update:model-value="onGroupingChange"
      />

      <v-text-field
        ref="searchRef"
        v-model="store.filter.search"
        placeholder="Поиск (ID, заявка, комментарий…)"
        variant="solo"
        density="compact"
        hide-details
        clearable
        min-width="150px"
        prepend-inner-icon="systemIcons:iconSearch"
        @keydown.enter="store.refreshQuery()"
        @click:clear="store.refreshQuery()"
      />
      <v-btn variant="tonal" height="90%" density="compact" @click="store.refreshQuery()">Найти</v-btn>
      <v-spacer />
      <v-btn color="primary" height="90%" variant="flat" density="compact" prepend-icon="systemIcons:iconPlus" @click="newTask">
        Новая задача
      </v-btn>
      <v-btn variant="tonal" height="90%" density="compact" prepend-icon="systemIcons:iconCog" @click="showSettings = true">
        Настройки
      </v-btn>
    </v-container>

    <v-main class="pa-4 d-flex flex-column">
      <v-alert v-if="store.error" type="error" density="compact" class="mb-3" closable @click:close="store.error = ''">
        {{ store.error }}
      </v-alert>

      <FilterBar />

      <TaskTable
        class="flex-grow-1"
        :rows="store.rows"
        :selected-ids="selectedIds"
        @selection-change="onSelectionChange"
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

    <TaskFormModal v-if="editing" :draft="editing" @save="onSaveTask" @close="editing = null" />
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
    <ReportModal v-if="showReport" @close="showReport = false" />
    <StatsModal v-if="showStats" @close="showStats = false" />
    <ImportExportModal v-if="showImportExport" @close="showImportExport = false" />
  </v-app>
</template>