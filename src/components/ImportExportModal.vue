<script setup>
import { ref, reactive, onMounted, onBeforeUnmount } from "vue";
import { save, open } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { refresh as refreshState, refreshQuery } from "../store";
import SvgIcon from "../icons/SvgIcon.vue";

const emit = defineEmits(["close"]);

const tab = ref("export");

const exportOpts = reactive({
  filename: "time_tracker_export.json",
  tasks: true,
  tags: true,
  clients: true,
  users: true,
});

const importOpts = reactive({
  path: "",
  tasks: true,
  tags: true,
  clients: true,
  users: true,
});

const busy = ref(false);
const error = ref("");
const notice = ref("");

async function doExport() {
  error.value = "";
  notice.value = "";
  busy.value = true;
  try {
    const path = await save({
      defaultPath: exportOpts.filename,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return;
    await api.exportJson(path, {
      tasks: exportOpts.tasks,
      tags: exportOpts.tags,
      clients: exportOpts.clients,
      users: exportOpts.users,
    });
    notice.value = `Экспортировано: ${path}`;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function pickFile() {
  error.value = "";
  notice.value = "";
  const path = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "JSON", extensions: ["json"] }],
  });
  if (path) importOpts.path = String(path);
}

async function doImport() {
  if (!importOpts.path) return;
  error.value = "";
  notice.value = "";
  busy.value = true;
  try {
    await api.importJson(importOpts.path, {
      tasks: importOpts.tasks,
      tags: importOpts.tags,
      clients: importOpts.clients,
      users: importOpts.users,
    });
    await refreshState();
    await refreshQuery();
    notice.value = `Импортировано из: ${importOpts.path}`;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

function onKeydown(e) {
  if (e.key === "Escape") emit("close");
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal modal-suit">
      <div class="modal-head">
        <span class="with-icon"><SvgIcon name="export" />Экспорт / Импорт</span>
        <button class="icon" @click="emit('close')"><SvgIcon name="close" /></button>
      </div>
      <div class="modal-body">
        <div class="tabs">
          <button :class="{ active: tab === 'export' }" @click="tab = 'export'">Экспорт</button>
          <button :class="{ active: tab === 'import' }" @click="tab = 'import'">Импорт</button>
        </div>

        <div v-if="error" class="banner-error">{{ error }}</div>
        <div v-if="notice" class="banner-ok">{{ notice }}</div>

        <template v-if="tab === 'export'">
          <div class="form-row">
            <label>Файл (по умолчанию)</label>
            <input v-model="exportOpts.filename" />
          </div>
          <div class="check-list">
            <label><input type="checkbox" v-model="exportOpts.tasks" /> Задачи</label>
            <label><input type="checkbox" v-model="exportOpts.tags" /> Теги</label>
            <label><input type="checkbox" v-model="exportOpts.clients" /> Клиенты</label>
            <label><input type="checkbox" v-model="exportOpts.users" /> Пользователи</label>
          </div>
          <button class="primary with-icon" :disabled="busy" @click="doExport"><SvgIcon name="export" />Экспорт…</button>
        </template>

        <template v-else>
          <div class="form-row">
            <label>Файл для импорта</label>
            <div class="file-row">
              <input :value="importOpts.path" placeholder="не выбран" readonly />
              <button class="small" @click="pickFile">Выбрать…</button>
            </div>
          </div>
          <div class="check-list">
            <label><input type="checkbox" v-model="importOpts.tasks" /> Задачи</label>
            <label><input type="checkbox" v-model="importOpts.tags" /> Теги</label>
            <label><input type="checkbox" v-model="importOpts.clients" /> Клиенты</label>
            <label><input type="checkbox" v-model="importOpts.users" /> Пользователи</label>
          </div>
          <button
            class="primary with-icon"
            :disabled="busy || !importOpts.path"
            @click="doImport"
          >
            <SvgIcon name="import" />Импортировать…
          </button>
        </template>
      </div>
      <div class="modal-foot">
        <button @click="emit('close')">Закрыть</button>
      </div>
    </div>
  </div>
</template>