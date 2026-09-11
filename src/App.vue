<script setup>
import { onMounted, onBeforeUnmount, reactive, ref, nextTick } from "vue";
import * as store from "./store";
import FilterBar from "./components/FilterBar.vue";
import TaskTable from "./components/TaskTable.vue";
import TaskFormModal from "./components/TaskFormModal.vue";
import SettingsModal from "./components/SettingsModal.vue";
import ReportModal from "./components/ReportModal.vue";
import StatsModal from "./components/StatsModal.vue";
import ImportExportModal from "./components/ImportExportModal.vue";

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
  searchRef.value?.select();
}

async function newTask() {
  if (!store.state.settings.username) {
    showSettings.value = true;
    return;
  }
  editing.value = {
    mode: "new",
    user: store.state.settings.username,
    order: "",
    tag: "",
    client: "",
    comment: "",
  };
}

function editTask(task) {
  editing.value = {
    mode: "edit",
    taskId: task.taskId,
    user: task.user,
    order: task.order,
    tag: task.tag,
    client: task.client,
    comment: task.comment,
  };
}

async function onSaveTask(draft) {
  if (draft.mode === "new") {
    await store.addTask(draft);
  } else {
    await store.updateTask(draft.taskId, draft);
  }
  editing.value = null;
  await nextTick(() => {});
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

function onGroupingChange(e) {
  store.applySettings(store.state.settings.username, e.target.value);
}

function onKeydown(e) {
  const tag = (e.target.tagName || "").toLowerCase();
  const typing = tag === "input" || tag === "textarea" || tag === "select";
  if (typing) {
    if (e.key === "Escape" && tag === "input" && e.target === searchRef.value) {
      store.state.filter.search = "";
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
  <div class="app">
    <div class="topbar">
      <h1>⏱ Time Tracker</h1>
      <button class="primary" @click="newTask">＋ Новая задача</button>
      <button @click="showSettings = true">⚙ Настройки</button>

      <div class="search-box">
        <label>
          Группировка:
          <select :value="store.state.settings.grouping" @change="onGroupingChange">
            <option value="none">Нет</option>
            <option value="day">По дням</option>
            <option value="client">По клиентам</option>
          </select>
        </label>
        <input
          ref="searchRef"
          v-model="store.state.filter.search"
          placeholder="🔍 Поиск (ID, заявка, комментарий…)"
          style="width: 280px"
          class="input"
          @keydown.enter="store.refreshQuery()"
        />
        <button class="small" @click="store.refreshQuery()">Найти</button>
      </div>
    </div>

    <div v-if="store.state.error" class="banner-error">{{ store.state.error }}</div>

    <div class="main">
      <FilterBar />

      <TaskTable
        :rows="store.state.rows"
        :selected-ids="selectedIds"
        @selection-change="onSelectionChange"
        @edit="editTask"
        @delete="onDeleteTasks"
      />

      <div class="footer">
        <div class="tags">
          <span>Задачи: <b>{{ store.state.totals.count }}</b></span>
          <span>Время: <b>{{ store.state.totals.timeLabel }}</b></span>
          <button @click="showReport = true">📄 Отчёт</button>
          <button @click="showStats = true">📊 Статистика</button>
          <button @click="showImportExport = true">⇅ Экспорт/Импорт</button>
        </div>
        <button v-if="selectedIds.size" class="danger" @click="onDeleteTasks(Array.from(selectedIds))">
          🗑 Удалить выбранное ({{ selectedIds.size }})
        </button>
      </div>
    </div>

    <TaskFormModal v-if="editing" :draft="editing" @save="onSaveTask" @close="editing = null" />
    <SettingsModal v-if="showSettings" @close="showSettings = false" />
    <ReportModal v-if="showReport" @close="showReport = false" />
    <StatsModal v-if="showStats" @close="showStats = false" />
    <ImportExportModal v-if="showImportExport" @close="showImportExport = false" />
  </div>
</template>