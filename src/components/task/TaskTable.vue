<script setup>
import { computed, defineModel } from "vue"
import { useAppStore } from "../../store.js"
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

const store = useAppStore()

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
])

const tableStyle = computed(() => ({
  fontFamily: store.settings.fontFamily || "Roboto",
  fontSize: (store.settings.fontSize || 14) + "px",
}))
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
      <task-group-header
        :item="item"
        :columns="columns"
        :toggle-group="toggleGroup"
        :is-group-open="isGroupOpen"
      />
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
      <task-our-car :task="item"></task-our-car>
    </template>

    <template #item.mode="{item}">
      <task-mode :task="item"/>
    </template>
    <template #item.order="{item}">
      <div class="order-element">{{item.order}}</div>
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

.order-element {
  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis; /* Добавляет многоточие в конце */
}

</style>