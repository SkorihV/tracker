<script setup>
import { computed, ref } from "vue"
import { useColumnsStore } from "../../stores/columns.js"
import { useSettingsStore } from "../../stores/settings.js"
import { useTasksStore } from "../../stores/tasks.js"
import { useColumnResize } from "../../composables/useColumnResize.js"
import { useRowSelection } from "../../composables/useRowSelection.js"
import { usePinnedBlock } from "../../composables/usePinnedBlock.js"
import { useScrollHints } from "../../composables/useScrollHints.js"
import TaskCell from "./items/TaskCell.vue"
import TaskGroupHeader from "./items/TaskGroupHeader.vue"
import TaskNoData from "./items/TaskNoData.vue"

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

const items = computed(() => tasksStore.unpinnedRows.map((r) => ({ ...r, _day: dayIso(r.start || "") })))

const groupBy = computed(() => {
  const g = settingsStore?.settings?.grouping
  if (g === "day") return [{ key: "_day", order: "desc" }]
  if (g === "client") return [{ key: "client", order: "asc" }]
  return []
})

const { rowProps, onRowClick } = useRowSelection(rows, selectedIds)

const wrapRef = ref(null)
const { hiddenAbove, hiddenBelow, onPinnedScroll, scrollPinned } = usePinnedBlock(wrapRef, () => [
  tasksStore.pinnedRows,
  columnsStore.visibleColumns,
  settingsStore.settings.fontSize,
])

const {
  above: rowsAbove,
  below: rowsBelow,
  topOffset: hintTop,
  bottomOffset: hintBottom,
  centerX: hintCenter,
  scrollMain,
} = useScrollHints(wrapRef, () => [tasksStore.unpinnedRows, tasksStore.pinnedRows, groupBy.value])

function editTask(_e, row) {
  emit("edit", row.item)
}

const tableHeaders = computed(() => [
  ...(groupBy.value.length ? [{ key: "data-table-group", title: "", sortable: false, width: 44 }] : []),
  { key: "pin", title: "", sortable: false, width: 72 },
  ...columnsStore.visibleColumns.map((c) => ({
    key: c.key,
    title: c.label,
    sortable: true,
    width: widths[c.key] != null ? clampWidth(c, widths[c.key]) : c.width || undefined,
    minWidth: c.minWidth || undefined,
    maxWidth: c.maxWidth || undefined,
  })),
])

// Колонки с собственными слотами ячеек (закрепление + выбранные пользователем).
const cellKeys = computed(() => ["pin", ...columnsStore.visibleColumns.map((c) => c.key)])
const slotName = (key) => `item.${key}`

const tableStyle = computed(() => ({
  fontFamily: settingsStore.settings.fontFamily || "Roboto",
  fontSize: (settingsStore.settings.fontSize || 14) + "px",
}))
</script>

<template>
  <div
    ref="wrapRef"
    class="task-table-wrap"
    @mousemove="onWrapperMousemove"
    @mousedown="onWrapperMousedown"
    @dblclick="onWrapperDblclick"
    @click.capture="onWrapperClickCapture"
  >
    <v-data-table
      :headers="tableHeaders"
      :items="items"
      item-value="taskId"
      density="compact"
      :style="tableStyle"
      hover
      class="task-table"
      fixed-header
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
      <template #header.pin>
        <v-btn
          variant="text"
          density="comfortable"
          size="small"
          class="pin-header-btn"
          :title="tasksStore.pinnedRows.length ? 'Сбросить закреплённые' : 'Закреплённых задач нет'"
          @click.stop="tasksStore.clearPins()"
        >
          <v-icon icon="mdi-paperclip" :color="tasksStore.pinnedRows.length ? 'primary' : undefined" />
          <span v-if="tasksStore.pinnedRows.length" class="ml-1 text-primary font-weight-bold">{{
            tasksStore.pinnedRows.length
          }}</span>
        </v-btn>
      </template>

      <template v-for="key in cellKeys" :key="key" #[slotName(key)]="{ item }">
        <task-cell :task="item" :column-key="key" @edit="emit('edit', $event)" @delete="emit('delete', $event)" />
      </template>

      <template #body.prepend="{ columns }">
        <tr v-if="tasksStore.pinnedRows.length" class="pinned-block">
          <td :colspan="columns.length">
            <div class="pinned-view">
              <div class="pinned-scroll" @scroll.passive="onPinnedScroll">
                <table class="pinned-table">
                  <colgroup>
                    <col v-for="col in columns" :key="col.key" />
                  </colgroup>
                  <tbody>
                    <tr
                      v-for="task in tasksStore.pinnedRows"
                      :key="task.taskId"
                      class="pinned-row"
                      :class="{ 'row-selected': selectedIds.has(task.taskId) }"
                      @click="onRowClick($event, { item: task })"
                      @dblclick="emit('edit', task)"
                    >
                      <td v-for="col in columns" :key="col.key">
                        <task-cell
                          v-if="col.key !== 'data-table-group'"
                          :task="task"
                          :column-key="col.key"
                          @edit="emit('edit', $event)"
                          @delete="emit('delete', $event)"
                        />
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <v-btn
                v-if="hiddenAbove"
                class="pinned-more pinned-more--top"
                size="x-small"
                color="primary"
                prepend-icon="mdi-chevron-up"
                title="Скрыто выше — прокрутить к началу"
                @click.stop="scrollPinned('top')"
              >
                {{ hiddenAbove }}
              </v-btn>
              <v-btn
                v-if="hiddenBelow"
                class="pinned-more pinned-more--bottom"
                size="x-small"
                color="primary"
                prepend-icon="mdi-chevron-down"
                title="Скрыто ниже — прокрутить к концу"
                @click.stop="scrollPinned('bottom')"
              >
                {{ hiddenBelow }}
              </v-btn>
            </div>
          </td>
        </tr>
      </template>

      <template #no-data>
        <task-no-data v-if="!tasksStore.pinnedRows.length" />
      </template>
    </v-data-table>
    <v-btn
      v-if="rowsAbove"
      class="scroll-hint"
      :style="{ top: `${hintTop}px`, left: `${hintCenter}px` }"
      size="x-small"
      color="primary"
      prepend-icon="mdi-chevron-up"
      title="Скрыто выше — прокрутить к началу"
      @click="scrollMain('top')"
    >
      {{ rowsAbove }}
    </v-btn>
    <v-btn
      v-if="rowsBelow"
      class="scroll-hint"
      :style="{ bottom: `${hintBottom}px`, left: `${hintCenter}px` }"
      size="x-small"
      color="primary"
      prepend-icon="mdi-chevron-down"
      title="Скрыто ниже — прокрутить к концу"
      @click="scrollMain('bottom')"
    >
      {{ rowsBelow }}
    </v-btn>
  </div>
</template>

<style lang="scss">
.task-table-wrap {
  position: relative;
  display: flex;
  flex-direction: column;
  flex: 1 1 0;
  min-height: 0;
  overflow: hidden;

  > .v-data-table {
    flex: 1 1 auto;
    min-height: 0;

    .v-table__wrapper {
      flex: 1 1 auto;
      min-height: 0;
      overflow: auto;
    }
  }
}

// Блок закреплённых задач: липнет под заголовком, высота — три строки, дальше своя прокрутка.
.task-table .pinned-block > td {
  position: sticky;
  top: var(--pin-top, 0);
  z-index: 1;
  padding: 0 !important;
  height: auto !important;
  background-color: rgb(var(--v-theme-surface));
}

.pinned-view {
  position: relative;
  // Ширина блока задаётся ячейкой таблицы, а не его содержимым: иначе вложенная таблица
  // растягивает основную, а та — вложенную, и ширина растёт без предела.
  contain: inline-size;
  background-color: rgb(var(--v-theme-surface));
  border-bottom: 2px solid rgba(var(--v-theme-primary), 0.6);
}

.pinned-scroll {
  max-height: 96px;
  overflow-x: hidden;
  overflow-y: auto;
  scrollbar-width: thin;
}

.pinned-table {
  width: 100%;
  table-layout: fixed;
  border-collapse: collapse;

  td {
    height: 32px;
    padding: 0 16px;
    overflow: hidden;
    text-align: start;
    border-bottom: thin solid rgba(var(--v-border-color), var(--v-border-opacity));
  }

  tr:last-child > td {
    border-bottom: 0;
  }
}

.pinned-row {
  cursor: pointer;

  &:hover > td {
    background-color: rgba(var(--v-theme-on-surface), 0.04);
  }

  &.row-selected > td {
    background-color: rgba(var(--v-theme-primary), 0.08);
  }
}

.pinned-more {
  position: absolute;
  left: var(--pin-center, 50%);
  z-index: 2;
  min-width: 0;
  transform: translateX(-50%);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
}

.pinned-more--top {
  top: 4px;
}

.pinned-more--bottom {
  bottom: 4px;
}

// Кнопки «скрыто выше/ниже» основной таблицы; стоят по центру области таблицы.
.scroll-hint {
  position: absolute;
  left: 50%;
  z-index: 3;
  min-width: 0;
  transform: translateX(-50%);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
}

.pin-header-btn {
  min-width: 0;
  padding: 0 6px;
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
