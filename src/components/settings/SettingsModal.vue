<script setup>
import { ref, computed } from "vue"
import { useAppStore } from "../../store.js"
import EntityPanel from "../EntityPanel.vue"
import ColumnsTab from "./tabs/ColumnsTab.vue"
import StatusesTab from "./tabs/StatusesTab.vue"
import GeneralTab from "./tabs/GeneralTab.vue"
import BaseModal from "../BaseModal.vue"

const store = useAppStore()
const emit = defineEmits(["close"])

const tab = ref("users")
const curUser = computed({
  get: () => store.settings.username,
  set: (v) => {
    if (v) store.applySettings(v, store.settings.grouping)
  },
})

async function onAction(kind, payload) {
  await store.saveEntity(kind, payload)
}
</script>

<template>
  <BaseModal title="Настройки" max-width="800" width="960" height="80vh" @close="emit('close')">
    <v-tabs v-model="tab" color="primary" class="mb-4">
          <v-tab value="general">Общее</v-tab>
          <v-tab value="users">Пользователи</v-tab>
          <v-tab value="tags">Теги</v-tab>
          <v-tab value="clients">Клиенты</v-tab>
          <v-tab value="statuses">Статусы</v-tab>
          <v-tab value="tasks">Заголовки таблицы</v-tab>
        </v-tabs>

        <div v-if="tab === 'general'">
          <GeneralTab />
        </div>
        <div v-if="tab === 'users'">
          <v-autocomplete
              v-model="curUser"
              :items="store.users.map((u) => ({ title: u, value: u }))"
              label="Текущий пользователь (для новых задач)"
              density="compact"
              variant="outlined"
              class="mb-4"
              style="max-width: 360px"
          />
          <EntityPanel kind="user" title="Пользователи" :items="store.users" @action="(p) => onAction('user', p)" />
        </div>
        <div v-if="tab === 'tags'">
          <EntityPanel kind="tag" title="Теги" :items="store.tags" @action="(p) => onAction('tag', p)" />
        </div>
        <div v-if="tab === 'clients'">
          <EntityPanel kind="client" title="Клиенты" :items="store.clients" @action="(p) => onAction('client', p)" />
        </div>
        <div v-if="tab === 'statuses'">
          <StatusesTab :items="store.statuses" @action="(p) => onAction('status', p)" />
        </div>
        <div v-if="tab === 'tasks'">
          <ColumnsTab />
        </div>

    <template #actions>
      <v-btn color="primary" variant="flat" @click="emit('close')">Закрыть</v-btn>
    </template>
  </BaseModal>
</template>