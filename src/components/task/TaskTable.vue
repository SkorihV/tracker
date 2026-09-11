<script setup>
import { ref, computed } from "vue";
import { useAppStore } from "../../store.js";
import { statusHex } from "../../statusColors.js";

const store = useAppStore();

const props = defineProps({
  rows: { type: Array, default: () => [] },
  selectedIds: { type: Object, default: () => new Set() },
});

const emit = defineEmits(["edit", "delete", "selection-change"]);

const localSelected = ref(new Set(props.selectedIds));

const allTaskIds = computed(() =>
  props.rows.filter((r) => r.kind === "task").map((r) => r.taskId)
);

const visibleColumns = computed(() => store.columns.filter((c) => c.visible));
const colSpan = computed(() => visibleColumns.value.length + 1);

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

const statusMeta = {
  running: { color: "success", icon: "systemIcons:iconPause" },
  paused: { color: "warning", icon: "systemIcons:iconPlay" },
  completed: { color: "secondary", icon: "systemIcons:iconStop" },
};

const statusLabel = {
  running: "Запущена",
  paused: "На паузе",
  completed: "Завершена",
};

const statusItems = computed(() =>
  store.statuses.map((s) => ({ title: s.name, value: s.name }))
);

const statusCellStyle = (row) => {
  const c = statusColorOf(row);
  return c ? `background-color: ${c};` : "";
};

function statusColorOf(row) {
  const st = store.statuses.find((s) => s.name === row.customStatus);
  return statusHex(st?.color);
}

async function onChangeStatus(row, value) {
  if (value === row.customStatus) return;
  await store.updateTaskStatus(row.taskId, value || "");
}
</script>

<template>
  <v-table density="compact" hover class="task-table">
    <thead>
      <tr>
        <th style="width: 44px">
          <v-checkbox
            :model-value="isAllSelected()"
            density="compact"
            hide-details
            @update:model-value="toggleAll"
          />
        </th>
        <th v-for="col in visibleColumns" :key="col.key">{{ col.label }}</th>
      </tr>
    </thead>
    <tbody>
      <template v-for="(row, i) in rows" :key="row.kind + '-' + (row.taskId || row.label) + '-' + i">
        <tr v-if="row.kind === 'group'" class="group-row">
          <td :colspan="colSpan">
            <div class="d-flex align-center justify-space-between">
              <div class="d-flex align-center">
                <v-icon size="small" class="mr-1">mdi-chevron-down</v-icon>
                <span class="font-weight-medium">{{ row.label }}</span>
                <span class="text-medium-emphasis ml-2">({{ row.count }} задач)</span>
              </div>
              <span class="mono font-weight-bold">{{ row.timeLabel }}</span>
            </div>
          </td>
        </tr>

        <tr
          v-else
          class="task-row"
          :class="'row-' + row.status"
          :title="row.taskId"
          @dblclick="emit('edit', row)"
          @click="toggleRow(row)"
        >
          <td>
            <v-checkbox
              :model-value="localSelected.has(row.taskId)"
              density="compact"
              hide-details
              @update:model-value="toggleRow(row)"
              @click.stop
            />
          </td>
          <td v-for="col in visibleColumns" :key="col.key">
            <template v-if="col.key === 'mode'">
              <v-chip :color="statusMeta[row.status]?.color || 'secondary'" size="small" variant="tonal">
                <template #prepend>
                  <v-icon :icon="statusMeta[row.status]?.icon || 'systemIcons:iconStop'" start />
                </template>
                <span class="text-caption">{{ statusLabel[row.status] || row.status }}</span>
              </v-chip>
            </template>
            <template v-else-if="col.key === 'status'">
              <v-sheet
                  height="100%"
                  width="100%"
                  min-width="150px"
                  class="pa-2 status-cell"
                  :style="statusCellStyle(row)"
                  @click.stop
              >
                <v-autocomplete
                    :model-value="row.customStatus"
                    :items="statusItems"
                    density="compact"
                    variant="plain"
                    hide-details

                    placeholder="Статус"
                    @update:model-value="onChangeStatus(row, $event)"
                />
              </v-sheet>
            </template>
            <template v-else-if="col.key === 'id'">
              <span class="mono">{{ row.taskId }}</span>
            </template>
            <template v-else-if="col.key === 'order'">
              {{ row.order }}
            </template>
            <template v-else-if="col.key === 'client'">
              {{ row.client }}
            </template>
            <template v-else-if="col.key === 'tag'">
              {{ (row.tags || []).join(", ") }}
            </template>
            <template v-else-if="col.key === 'start'">
              <span class="mono">{{ row.startLabel }}</span>
            </template>
            <template v-else-if="col.key === 'end'">
              <span class="mono">{{ row.endLabel || "—" }}</span>
            </template>
            <template v-else-if="col.key === 'time'">
              <span class="mono font-weight-bold">{{ row.timeLabel }}</span>
            </template>
            <template v-else-if="col.key === 'user'">
              {{ row.user }}
            </template>
            <template v-else-if="col.key === 'comment'">
              <div class="comment-cell">{{ row.comment }}</div>
            </template>
            <template v-else-if="col.key === 'actions'">
              <div class="d-flex ga-1">
                <template v-if="row.status === 'running'">
                  <v-btn icon aria-label="Пауза" variant="text" size="small" @click.stop="onPause(row)">
                    <v-icon icon="systemIcons:iconPause" color="orange" />
                  </v-btn>
                  <v-btn icon aria-label="Завершить" variant="text" size="small" @click.stop="onComplete(row)">
                    <v-icon icon="systemIcons:iconStop"  color="red"/>
                  </v-btn>
                </template>
                <template v-else-if="row.status === 'paused'">
                  <v-btn icon aria-label="Возобновить"  variant="text" size="small" @click.stop="onStart(row)">
                    <v-icon icon="systemIcons:iconPlay" color="green"/>
                  </v-btn>
                  <v-btn icon aria-label="Завершить" variant="text" size="small" @click.stop="onComplete(row)">
                    <v-icon icon="systemIcons:iconStop"color="red" />
                  </v-btn>
                </template>
                <template v-else>
                  <v-btn icon aria-label="Возобновить" variant="text" size="small" @click.stop="onStart(row)">
                    <v-icon icon="systemIcons:iconPlay" color="green"/>
                  </v-btn>
                </template>
                <v-btn icon aria-label="Редактировать" variant="text" size="small" @click.stop="emit('edit', row)">
                  <v-icon icon="systemIcons:iconEdit" />
                </v-btn>
                <v-btn icon aria-label="Удалить" variant="text" size="small" color="error" @click.stop="onDeleteOne(row)">
                  <v-icon icon="systemIcons:iconTrash" />
                </v-btn>
              </div>
            </template>
          </td>
        </tr>
      </template>
      <tr v-if="!rows.length">
        <td class="empty" :colspan="colSpan">Нет задач</td>
      </tr>
    </tbody>
  </v-table>
</template>

<style lang="scss">
.status-cell .v-autocomplete__selection-text {
  border-radius: 6px;
  color: #fff;
}
</style>