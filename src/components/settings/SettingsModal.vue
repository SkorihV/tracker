<script setup>
import { ref, computed } from "vue"
import { useDictionariesStore } from "../../stores/dictionaries.js"
import { useSettingsStore } from "../../stores/settings.js"
import TabEntity from "./tabs/TabEntity.vue"
import TabColumns from "./tabs/TabColumns.vue"
import TabStatuses from "./tabs/TabStatuses.vue"
import TabGeneral from "./tabs/TabGeneral.vue"
import BaseModal from "../base/BaseModal.vue"

const dicts = useDictionariesStore()
const settingsStore = useSettingsStore()
const emit = defineEmits(["close"])

const tab = ref("users")
const curUser = computed({
  get: () => settingsStore.settings.username,
  set: (v) => {
    if (v) settingsStore.applySettings(v, settingsStore.settings.grouping)
  },
})

async function onAction(kind, payload) {
  await dicts.saveEntity(kind, payload)
}
</script>

<template>
  <BaseModal title="Настройки" max-width="800" width="960" height="80vh" @close="emit('close')">
    <template #header>
      <v-tabs v-model="tab" color="primary" class="mb-4">
        <v-tab value="general">Общее</v-tab>
        <v-tab value="users">Пользователи</v-tab>
        <v-tab value="tags">Теги</v-tab>
        <v-tab value="clients">Клиенты</v-tab>
        <v-tab value="statuses">Статусы</v-tab>
        <v-tab value="tasks">Заголовки таблицы</v-tab>
      </v-tabs>
    </template>

    <div v-if="tab === 'general'">
      <TabGeneral />
    </div>
    <div v-if="tab === 'users'">
      <v-autocomplete
        v-model="curUser"
        :items="dicts.users.map((u) => ({ title: u, value: u }))"
        label="Текущий пользователь (для новых задач)"
        density="compact"
        variant="outlined"
        class="mb-4"
        style="max-width: 360px"
      />
      <TabEntity kind="user" title="Пользователи" :items="dicts.users" @action="(p) => onAction('user', p)" />
    </div>
    <div v-if="tab === 'tags'">
      <TabEntity kind="tag" title="Теги" :items="dicts.tags" @action="(p) => onAction('tag', p)" />
    </div>
    <div v-if="tab === 'clients'">
      <TabEntity kind="client" title="Клиенты" :items="dicts.clients" @action="(p) => onAction('client', p)" />
    </div>
    <div v-if="tab === 'statuses'">
      <TabStatuses :items="dicts.statuses" @action="(p) => onAction('status', p)" />
    </div>
    <div v-if="tab === 'tasks'">
      <TabColumns />
    </div>

    <template #actions>
      <v-btn color="primary" variant="flat" @click="emit('close')">Закрыть</v-btn>
    </template>
  </BaseModal>
</template>
