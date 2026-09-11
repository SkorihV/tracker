<script setup>
import { ref, computed } from "vue";
import * as store from "../store";
import SvgIcon from "../icons/SvgIcon.vue";

const props = defineProps({
  rows: { type: Array, default: () => [] },
  selectedIds: { type: Object, default: () => new Set() },
});

const emit = defineEmits(["edit", "delete", "selection-change"]);

const localSelected = ref(new Set(props.selectedIds));

const allTaskIds = computed(() =>
  props.rows.filter((r) => r.kind === "task").map((r) => r.taskId)
);

function isAllSelected() {
  const ids = allTaskIds.value;
  return ids.length > 0 && ids.every((id) => localSelected.value.has(id));
}

function sync() {
  emit("selection-change", new Set(localSelected.value));
}

function toggleRow(task) {
  const next = new Set(localSelected.value);
  if (next.has(task.taskId)) next.delete(task.taskId);
  else next.add(task.taskId);
  localSelected.value = next;
  sync();
}

function toggleAll() {
  const next = new Set(localSelected.value);
  if (isAllSelected()) {
    for (const id of allTaskIds.value) next.delete(id);
  } else {
    for (const id of allTaskIds.value) next.add(id);
  }
  localSelected.value = next;
  sync();
}

async function onStart(task) {
  if (task.status === "running") return;
  if (task.status === "paused") await store.resumeTask(task.taskId);
  else await store.startTask(task.taskId);
}

async function onPause(task) {
  if (task.status === "running") await store.pauseTask(task.taskId);
}

async function onComplete(task) {
  await store.completeTask(task.taskId);
}

function onDeleteOne(task) {
  emit("delete", [task.taskId]);
}
</script>

<template>
  <div class="table-wrap">
    <table class="grid">
      <thead>
        <tr>
          <th style="width: 32px">
            <input type="checkbox" :checked="isAllSelected()" @change="toggleAll" />
          </th>
          <th>Статус</th>
          <th>ID</th>
          <th>Заявка</th>
          <th>Клиент</th>
          <th>Тег</th>
          <th>Начало</th>
          <th>Завершение</th>
          <th>Время</th>
          <th>Пользователь</th>
          <th style="min-width: 180px">Комментарий</th>
          <th>Действия</th>
        </tr>
      </thead>
      <tbody>
        <template v-for="(row, i) in rows" :key="row.kind + '-' + (row.taskId || row.label) + '-' + i">
          <tr v-if="row.kind === 'group'" class="group">
            <td colspan="3">▼ {{ row.label }}</td>
            <td colspan="1">{{ row.count }} задач</td>
            <td colspan="7"></td>
            <td class="mono">{{ row.timeLabel }}</td>
            <td></td>
          </tr>

          <tr
            v-else class="task"
              @dblclick="$emit('edit', row)"
            @click="toggleRow(row)"
              :class="row.status" :title="row.taskId">
            <td>
              <input
                type="checkbox"
                :checked="localSelected.has(row.taskId)"
                @change="toggleRow(row)"
                @click.stop
              />
            </td>
            <td>
              <span class="status-badge" :class="'status-' + row.status">
                <SvgIcon :name="row.status === 'running' ? 'pause' : row.status === 'paused' ? 'play' : 'stop'" />
              </span>
            </td>
            <td class="mono">{{ row.taskId }}</td>
            <td>{{ row.order }}</td>
            <td>{{ row.client }}</td>
            <td>{{ row.tag }}</td>
            <td class="mono">{{ row.startLabel }}</td>
            <td class="mono">{{ row.endLabel || "—" }}</td>
            <td class="mono"><b>{{ row.timeLabel }}</b></td>
            <td>{{ row.user }}</td>
            <td class="mono" style="white-space: normal; max-width: 240px">
              {{ row.comment }}
            </td>
            <td>
              <div class="cell-actions">
                <template v-if="row.status === 'running'">
                  <button class="small with-icon" @click.stop="onPause(row)"><SvgIcon name="pause" /></button>
                  <button class="small with-icon" @click.stop="onComplete(row)"><SvgIcon name="stop" /></button>
                </template>
                <template v-else-if="row.status === 'paused'">
                  <button class="small with-icon" @click.stop="onStart(row)"><SvgIcon name="play" /></button>
                  <button class="small with-icon" @click.stop="onComplete(row)"><SvgIcon name="stop" /></button>
                </template>
                <template v-else>
                  <button class="small with-icon" @click.stop="onStart(row)"><SvgIcon name="play" /></button>
                </template>
                <button class="small with-icon" @click.stop="$emit('edit', row)"><SvgIcon name="edit" /></button>
                <button class="small danger with-icon" @click.stop="onDeleteOne(row)"><SvgIcon name="trash" /></button>
              </div>
            </td>
          </tr>
        </template>
        <tr v-if="!rows.length">
          <td class="empty" colspan="12">Нет задач</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>