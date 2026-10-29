<script setup>
import { computed } from "vue"
import { useColumnsStore } from "../../stores/columns.js"
import { useSettingsStore } from "../../stores/settings.js"
import { useTasksStore } from "../../stores/tasks.js"
import { useColumnResize } from "../../composables/useColumnResize.js"
import { useRowSelection } from "../../composables/useRowSelection.js"
import TaskActions from "./items/TaskActions.vue"
import TaskComment from "./items/TaskComment.vue"
import TaskTime from "./items/TaskTime.vue"
import TaskStart from "./items/TaskStart.vue"
import TaskEnd from "./items/TaskEnd.vue"
import TaskTags from "./items/TaskTags.vue"
import TaskStatus from "./items/TaskStatus.vue"
import TaskOurCar from "./items/TaskOurCar.vue"
import TaskGroupHeader from "./items/TaskGroupHeader.vue"
import TaskNoData from "./items/TaskNoData.vue"
import TaskMode from "./items/TaskMode.vue"

const columnsStore = useColumnsStore()
const settingsStore = useSettingsStore()
const tasksStore = useTasksStore()

const selectedIds = defineModel("selectedIds", { default: new Set() })

const emit = defineEmits(["edit", "delete"])

const rows = computed(() => tasksStore.rows)

// Ресайз разрешён только для этих колонок
const RESIZABLE_KEYS = ["order", "comment", "client", "tags"]

const {
  widths,
  clampWidth,
  onMousemove: onWrapperMousemove,
  onMousedown: onWrapperMousedown,
  onDblclick: onWrapperDblclick,
  onClickCapture: onWrapperClickCapture,
} = useColumnResize({
  getHeaders: () => tableHeaders.value,
  getColumns: () => columnsStore.visibleColumns,
  resizableKeys: RESIZABLE_KEYS,
})

function dayIso(label) {
  const m = /^(\d{2})\.(\d{2})\.(\d{4})/.exec(label || "")
  return m ? `${m[3]}-${m[2]}-${m[1]}` : ""
}

const items = computed(() => rows.value.map((r) => ({ ...r, _day: dayIso(r.start || "") })))

const groupBy = computed(() => {
  const g = settingsStore?.settings?.grouping
  if (g === "day") return [{ key: "_day", order: "desc" }]
  if (g === "client") return [{ key: "client", order: "asc" }]
  return []
})

const { selectedArray, rowProps, onRowClick } = useRowSelection(rows, selectedIds)

function editTask(_e, row) {
  emit("edit", row.item)
}

const tableHeaders = computed(() => [
  ...(groupBy.value.length ? [{ key: "data-table-group", title: "", sortable: false, width: 44 }] : []),
  { key: "data-table-select", title: "", sortable: false, width: 44 },
  ...columnsStore.visibleColumns.map((c) => ({
    key: c.key,
    title: c.label,
    sortable: true,
    width: widths[c.key] != null ? clampWidth(c, widths[c.key]) : c.width || undefined,
    minWidth: c.minWidth || undefined,
    maxWidth: c.maxWidth || undefined,
  })),
])

const tableStyle = computed(() => ({
  fontFamily: settingsStore.settings.fontFamily || "Roboto",
  fontSize: (settingsStore.settings.fontSize || 14) + "px",
}))
</script>

<template>
  <div
    class="task-table-wrap"
    @mousemove="onWrapperMousemove"
    @mousedown="onWrapperMousedown"
    @dblclick="onWrapperDblclick"
    @click.capture="onWrapperClickCapture"
  >
    <v-data-table
      v-model="selectedArray"
      :headers="tableHeaders"
      :items="items"
      item-value="taskId"
      density="compact"
      :style="tableStyle"
      hover
      class="task-table"
      hide-default-footer
      :items-per-page="-1"
      no-filter
      :group-by="groupBy"
      open-all
      :row-props="rowProps"
      @click:row="onRowClick"
      @dblclick:row="editTask"
    >
      <template #group-header="{ item, columns, toggleGroup, isGroupOpen }">
        <task-group-header :item="item" :columns="columns" :toggle-group="toggleGroup" :is-group-open="isGroupOpen" />
      </template>
      <template #header.data-table-group>
        <v-icon size="small">mdi-view-agenda</v-icon>
      </template>
      <template #colgroup="{ columns }">
        <colgroup>
          <col
            v-for="col in columns"
            :key="col.key"
            :style="col?.width ? { width: typeof col.width === 'number' ? col.width + 'px' : col.width } : undefined"
          />
        </colgroup>
      </template>
      <template #header.ourCar>
        <div class="d-flex align-center justify-center w-100">
          <v-icon icon="mdi-car" title="Наша машина" />
        </div>
      </template>
      <template #header.data-table-select="{ someSelected, allSelected, selectAll }">
        <v-checkbox
          :model-value="allSelected"
          :indeterminate="someSelected && !allSelected"
          density="compact"
          hide-details
          :color="settingsStore.settings.accentColor"
          @update:model-value="selectAll"
        />
      </template>
      <template #item.data-table-select="{ props: cellProps }">
        <v-checkbox-btn v-bind="cellProps" density="compact" :color="settingsStore.settings.accentColor" />
      </template>
      <template #item.actions="{ item }">
        <task-actions :task="item" @delete="emit('delete', $event)" @edit="emit('edit', $event)"></task-actions>
      </template>
      <template #item.comment="{ item }">
        <task-comment :task="item"></task-comment>
      </template>
      <template #item.time="{ item }">
        <task-time :task="item"></task-time>
      </template>
      <template #item.start="{ item }">
        <task-start :task="item"></task-start>
      </template>
      <template #item.end="{ item }">
        <task-end :task="item"></task-end>
      </template>
      <template #item.tags="{ item }">
        <task-tags :task="item"></task-tags>
      </template>
      <template #item.status="{ item }">
        <task-status :task="item"></task-status>
      </template>

      <template #item.ourCar="{ item }">
        <task-our-car :task="item"></task-our-car>
      </template>

      <template #item.mode="{ item }">
        <task-mode :task="item" />
      </template>
      <template #item.order="{ item }">
        <div class="order-element">{{ item.order }}</div>
      </template>

      <template #no-data>
        <task-no-data />
      </template>
    </v-data-table>
  </div>
</template>

<style lang="scss">
.task-table-wrap {
  display: flex;
  flex-direction: column;
  min-height: 0;

  > .v-data-table {
    flex: 1 1 auto;
    min-height: 0;
  }
}

.task-table .row-selected > td {
  background-color: rgba(var(--v-theme-primary), 0.08);
}

.task-table .v-table__wrapper > table {
  width: max-content;
  min-width: 100%;
}

// Зона ресайза колонки: подсветка правого края th при наведении
.task-table thead th {
  position: relative;
}

.task-table thead th.col-resize-zone::after {
  content: "";
  position: absolute;
  right: -2px;
  top: 0;
  bottom: 0;
  width: 5px;
  border-radius: 2px;
  background: rgba(var(--v-theme-primary), 0.35);
  cursor: col-resize;
}

.task-table thead th.col-resize-active::after {
  background: rgb(var(--v-theme-primary));
}

body.tt-col-resizing {
  cursor: col-resize !important;
  user-select: none;
}

.order-element {
  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis; /* Добавляет многоточие в конце */
}
</style>
