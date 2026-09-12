<script setup>
import { computed, defineModel } from "vue";
import { useAppStore } from "../../store.js";
import { fmtTd } from "../../api.js";
import TaskActions from "./TaskActions.vue";
import TaskComment from "./TaskComment.vue";
import TaskTime from "./TaskTime.vue";
import TaskStart from "./TaskStart.vue";
import TaskEnd from "./TaskEnd.vue";
import TaskTags from "./TaskTags.vue";
import TaskStatus from "./TaskStatus.vue";
import { statusHex, statusTextHex } from "../../statusColors.js";
import TaskNoData from "./TaskNoData.vue";
import TaskMode from "./TaskMode.vue";

const store = useAppStore();

const selectedIds = defineModel('selectedIds', {default: new Set()})

const emit = defineEmits(["edit", "delete"])

const rows = computed(() => store?.rows ?? [])

function dayIso(label) {
  const m = /^(\d{2})\.(\d{2})\.(\d{4})/.exec(label || "")
  return m ? `${m[3]}-${m[2]}-${m[1]}` : ""
}

const items = computed(() =>
  rows.value.map((r) => ({ ...r, _day: dayIso(r.start || "") }))
)

const groupBy = computed(() => {
  const g = store?.settings?.grouping
  if (g === "day") return [{ key: "_day", order: "desc" }]
  if (g === "client") return [{ key: "client", order: "asc" }]
  return []
})

const selectedArray = computed({
  get: () => Array.from(selectedIds.value || new Set()),
  set: (v) => { selectedIds.value = new Set(v || []) },
})

function groupLabel(item) {
  if (item.key === "_day") {
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(item.value || "")
    return m ? `${m[3]}.${m[2]}.${m[1]}` : item.value || "—"
  }
  return item.value || "(не указан)"
}

function groupTime(item) {
  const tasks = (item.items || []).map((x) => (x && x.raw ? x.raw : x))
  const total = tasks.reduce((s, t) => s + (t.seconds || 0), 0)
  return { count: tasks.length, time: fmtTd(total) }
}

function editTask(_e, row) {
  emit('edit', row.item)
}

function rowProps({ internalItem }) {
  const id = internalItem?.raw?.taskId
  return selectedIds.value.has(id) ? { class: 'row-selected' } : {}
}



function onRowClick(event, { item }) {
  const id = item?.taskId
  if (id == null) return

  if (event.target instanceof Element && event.target.closest('input, textarea, select, .v-btn, .v-select, .v-autocomplete, .v-menu, .v-list-item')) return

  const next = new Set(selectedIds.value || new Set())

  if (event.shiftKey && rows.value.length) {
    const visible = rows.value.filter((r) => r.taskId != null).map((r) => r.taskId)
    const anchor = visible.find((x) => next.has(x)) ?? id
    const from = visible.indexOf(anchor)
    const to = visible.indexOf(id)
    if (from >= 0 && to >= 0) {
      const [lo, hi] = [Math.min(from, to), Math.max(from, to)]
      for (let i = lo; i <= hi; i++) next.add(visible[i])
      selectedIds.value = next
    }
    return
  }

  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

const tableHeaders = computed(() => [
  ...(groupBy.value.length ? [{ key: "data-table-group", title: "", sortable: false, width: 44 }] : []),
  { key: "data-table-select", title: "", sortable: false, width: 44 },
  ...store.visibleColumns.map((c) => ({
    key: c.key,
    title: c.label,
    sortable: true,
    width: c.width || undefined
  })),
]);

const tableStyle = computed(() => ({
  fontFamily: store.settings.fontFamily || "Roboto",
  fontSize: (store.settings.fontSize || 14) + "px",
}));
</script>

<template>

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
      <tr class="task-group-row">
        <td :colspan="columns.length">
          <div class="d-flex align-center ga-1">
            <v-btn icon size="x-small" variant="text" density="default" @click="toggleGroup(item)">
              <v-icon size="x-large">{{ isGroupOpen(item) ? 'mdi-chevron-down' : 'mdi-chevron-right' }}</v-icon>
            </v-btn>
            <span class="font-weight-bold text-body-2">{{ groupLabel(item) }}</span>
            <span class="text-medium-emphasis text-caption ml-1">
              {{ groupTime(item).count }} задач · {{ groupTime(item).time }}
            </span>
          </div>
        </td>
      </tr>
    </template>
    <template #header.data-table-group>
      <v-icon size="small">mdi-view-agenda</v-icon>
    </template>
    <template #colgroup="{ columns }">
      <colgroup>
        <col v-for="col in columns" :key="col.key" :style="col?.width ? { width: typeof col.width === 'number' ? col.width + 'px' : col.width } : undefined" />
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
        :color="store.settings.accentColor"
        @update:model-value="selectAll"
      />
    </template>
    <template #item.data-table-select="{ item, props: cellProps }">
      <v-checkbox-btn
        v-bind="cellProps"
        density="compact"
        :color="store.settings.accentColor"
      />
    </template>
    <template #item.actions="{item}">
        <task-actions
            :task="item"
            @delete="emit('delete', $event)"
            @edit="emit('edit', $event)"
        ></task-actions>
    </template>
    <template #item.comment="{item}">
        <task-comment :task="item"></task-comment>
    </template>
    <template #item.time="{item}">
        <task-time :task="item"></task-time>
    </template>
    <template #item.start="{item}">
      <task-start :task="item"></task-start>
    </template>
    <template #item.end="{item}">
      <task-end :task="item"></task-end>
    </template>
    <template #item.tags="{item}">
      <task-tags :task="item"></task-tags>
    </template>
    <template #item.status="{item}">
      <task-status :task="item"></task-status>
    </template>

    <template #item.ourCar="{item}">
      <v-sheet
        v-if="item.ourCar"
        height="100%"
        width="100%"
        min-width="60px"
        class="pa-2 our-car-cell d-flex align-center justify-center"
        :style="{ backgroundColor: statusHex(store.settings.ourCarColor), color: statusTextHex(store.settings.ourCarColor) }"
      >
        <v-icon icon="mdi-car" title="Наша машина" />
      </v-sheet>
    </template>

    <template #item.mode="{item}">
      <task-mode :task="item"/>
    </template>

    <template #no-data>
      <task-no-data/>
    </template>
  </v-data-table>
</template>

<style lang="scss">

.task-table .row-selected > td {
  background-color: rgba(var(--v-theme-primary), 0.08);
}

.task-table .v-table__wrapper > table {
  width: max-content;
  min-width: 100%;
}

.task-table .task-group-row td {
  background: #eceef2;
  font-weight: 600;
}
</style>