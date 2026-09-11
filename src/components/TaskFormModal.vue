<script setup>
import { reactive, computed } from "vue";
import { state } from "../store";
import SvgIcon from "../icons/SvgIcon.vue";

const props = defineProps({
  draft: { type: Object, required: true },
});

const emit = defineEmits(["save", "close"]);

const form = reactive({
  mode: props.draft.mode,
  taskId: props.draft.taskId || "",
  user: props.draft.user || state.settings.username || "",
  order: props.draft.order || "",
  tag: props.draft.tag || "",
  client: props.draft.client || "",
  comment: props.draft.comment || "",
});

const title = computed(() =>
  form.mode === "new" ? "Новая задача" : `Задача ${form.taskId}`
);

function submit() {
  emit("save", { ...form });
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal" role="dialog" aria-modal="true">
      <div class="modal-head">
        <span>{{ title }}</span>
        <button class="icon" @click="emit('close')"><SvgIcon name="close" /></button>
      </div>
      <div class="modal-body">
        <div class="form-grid2">
          <div class="form-row">
            <label class="with-icon"><SvgIcon name="user" />Пользователь</label>
            <select v-model="form.user">
              <option v-for="u in state.users" :key="u" :value="u">{{ u }}</option>
            </select>
          </div>
          <div class="form-row">
            <label>Номер заявки (заказ)</label>
            <input v-model="form.order" placeholder="например 12345" />
          </div>
          <div class="form-row">
            <label>Тег</label>
            <select v-model="form.tag">
              <option value=""></option>
              <option v-for="t in state.tags" :key="t.id" :value="t.name">{{ t.name }}</option>
            </select>
          </div>
          <div class="form-row">
            <label class="with-icon"><SvgIcon name="client" />Клиент</label>
            <select v-model="form.client">
              <option value=""></option>
              <option v-for="c in state.clients" :key="c.id" :value="c.name">{{ c.name }}</option>
            </select>
          </div>
        </div>
        <div class="form-row">
          <label>Комментарий</label>
          <textarea
            v-model="form.comment"
            rows="3"
            placeholder="Комментарий к задаче"
            @keydown.esc="emit('close')"
          ></textarea>
        </div>
      </div>
      <div class="modal-foot">
        <button @click="emit('close')">Отмена</button>
        <button class="primary" @click="submit">Сохранить</button>
      </div>
    </div>
  </div>
</template>