<script setup>
import { ref, onMounted, onBeforeUnmount } from "vue";
import { state, saveEntity, applySettings } from "../store";
import EntityPanel from "./EntityPanel.vue";
import SvgIcon from "../icons/SvgIcon.vue";

const emit = defineEmits(["close"]);

const curUser = ref(state.settings.username);

async function onAction(kind, payload) {
  await saveEntity(kind, payload);
}

async function onUserChange() {
  await applySettings(curUser.value, state.settings.grouping);
}

function onKeydown(e) {
  if (e.key === "Escape") emit("close");
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal" style="min-width: 820px">
      <div class="modal-head">
        <span class="with-icon"><SvgIcon name="cog" />Настройки</span>
        <button class="icon" @click="emit('close')"><SvgIcon name="close" /></button>
      </div>
      <div class="modal-body">
        <div class="form-row" style="max-width: 320px">
          <label>Текущий пользователь (для новых задач)</label>
          <select v-model="curUser" @change="onUserChange">
            <option v-for="u in state.users" :key="u" :value="u">{{ u }}</option>
          </select>
        </div>
        <div class="settings-grid">
          <EntityPanel
            kind="user"
            title="Пользователи"
            :items="state.users"
            @action="(p) => onAction('user', p)"
          />
          <EntityPanel
            kind="tag"
            title="Теги"
            :items="state.tags"
            @action="(p) => onAction('tag', p)"
          />
          <EntityPanel
            kind="client"
            title="Клиенты"
            :items="state.clients"
            @action="(p) => onAction('client', p)"
          />
        </div>
      </div>
      <div class="modal-foot">
        <button class="primary" @click="emit('close')">Закрыть</button>
      </div>
    </div>
  </div>
</template>