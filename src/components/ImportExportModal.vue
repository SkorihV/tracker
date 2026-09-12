<script setup>
import { ref, reactive, computed } from "vue";
import { save, open } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { useAppStore } from "../store";

const store = useAppStore();
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
    await store.refresh();
    await store.refreshQuery();
    notice.value = `Импортировано из: ${importOpts.path}`;
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

const dialogOpen = computed({
  get: () => true,
  set: () => emit("close"),
});
</script>

<template>
  <v-dialog v-model="dialogOpen" max-width="560" max-height="85vh">
    <v-card class="modal-card">
      <v-toolbar v-dialog-drag density="compact" color="primary">
        <v-toolbar-title>Экспорт / Импорт</v-toolbar-title>
        <v-spacer />
        <v-btn icon variant="text" title="Закрыть" @click="emit('close')">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-toolbar>

      <v-card-text class="pt-5">
        <v-tabs v-model="tab" color="primary" class="mb-4">
          <v-tab value="export">Экспорт</v-tab>
          <v-tab value="import">Импорт</v-tab>
        </v-tabs>

        <v-alert v-if="error" type="error" density="compact" variant="tonal" class="mb-3">{{ error }}</v-alert>
        <v-alert v-if="notice" type="success" density="compact" variant="tonal" class="mb-3">{{ notice }}</v-alert>

        <template v-if="tab === 'export'">
          <v-text-field
            v-model="exportOpts.filename"
            label="Файл (по умолчанию)"
            density="compact"
            variant="outlined"
            class="mb-3"
          />
          <v-checkbox v-model="exportOpts.tasks" label="Задачи" density="compact" hide-details />
          <v-checkbox v-model="exportOpts.tags" label="Теги" density="compact" hide-details />
          <v-checkbox v-model="exportOpts.clients" label="Клиенты" density="compact" hide-details />
          <v-checkbox v-model="exportOpts.users" label="Пользователи" density="compact" hide-details />
        </template>

        <template v-else>
          <v-text-field
            :model-value="importOpts.path"
            label="Файл для импорта"
            placeholder="не выбран"
            density="compact"
            variant="outlined"
            readonly
            class="mb-2"
            :append-inner-icon="importOpts.path ? 'systemIcons:iconClose' : undefined"
            @click:append-inner="importOpts.path = ''"
          />
          <v-btn variant="tonal" size="small" prepend-icon="systemIcons:iconImport" class="mb-3" @click="pickFile">
            Выбрать…
          </v-btn>
          <v-checkbox v-model="importOpts.tasks" label="Задачи" density="compact" hide-details />
          <v-checkbox v-model="importOpts.tags" label="Теги" density="compact" hide-details />
          <v-checkbox v-model="importOpts.clients" label="Клиенты" density="compact" hide-details />
          <v-checkbox v-model="importOpts.users" label="Пользователи" density="compact" hide-details />
        </template>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn variant="text" @click="emit('close')">Закрыть</v-btn>
        <template v-if="tab === 'export'">
          <v-btn color="primary" variant="flat" prepend-icon="systemIcons:iconExport" :disabled="busy" @click="doExport">
            Экспорт…
          </v-btn>
        </template>
        <template v-else>
          <v-btn color="primary" variant="flat" prepend-icon="systemIcons:iconImport" :disabled="busy || !importOpts.path" @click="doImport">
            Импортировать…
          </v-btn>
        </template>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>