<script setup>
import { computed, defineModel } from "vue";
import { useAppStore } from "../../store.js";
import TaskActions from "./TaskActions.vue";
import TaskComment from "./TaskComment.vue";
import TaskTime from "./TaskTime.vue";
import TaskStart from "./TaskStart.vue";
import TaskEnd from "./TaskEnd.vue";
import TaskTags from "./TaskTags.vue";
import TaskStatus from "./TaskStatus.vue";
import TaskNoData from "./TaskNoData.vue";
import TaskMode from "./TaskMode.vue";

const store = useAppStore();

const selectedIds = defineModel('selectedIds', {default: new Set()})

const emit = defineEmits(["edit", "delete"])

const rows = computed(() => store?.rows ?? [])

const selectedArray = computed({
  get: () => Array.from(selectedIds.value || new Set()),
  set: (v) => { selectedIds.value = new Set(v || []) },
})

const groupBy =computed(() => {
 return [{ key: 'type', order: 'asc' }]
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

  if (event.ctrlKey || event.metaKey) {
    if (next.has(id)) next.delete(id)
    else next.add(id)
  } else {
    next.add(id)
  }
  selectedIds.value = next
}

const tableHeaders = computed(() => [
  { key: "data-table-select", title: "", sortable: false, width: 44 },
  ...store.visibleColumns.map((c) => ({
    key: c.key,
    title: c.label,
    sortable: true,
    width: c.width || undefined
  })),
]);
</script>

<template>
  <v-data-table
    v-model="selectedArray"
    :headers="tableHeaders"
    :items="rows"
    item-value="taskId"
    density="compact"

    hover
    class="task-table"
    hide-default-footer
    :items-per-page="-1"
    no-filter
    :row-props="rowProps"
    @click:row="onRowClick"
    @dblclick:row="editTask"
  >
    <template #colgroup="{ columns }">
      <colgroup>
        <col v-for="col in columns" :key="col.key" :style="col?.width ? { width: typeof col.width === 'number' ? col.width + 'px' : col.width } : undefined" />
      </colgroup>
    </template>
    <template #header.data-table-select="{ someSelected, allSelected, selectAll }">
      <v-checkbox
        :model-value="allSelected"
        :indeterminate="someSelected && !allSelected"
        density="compact"
        hide-details
        @update:model-value="selectAll"
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

    <template #item.mode="{item}">
      <task-mode :task="item"/>
    </template>

    <template #no-data>
      <task-no-data/>
    </template>
  </v-data-table>
</template>

<style lang="scss">
.status-cell .v-autocomplete__selection-text {
  border-radius: 6px;
  color: #fff;
}

.task-table .row-selected > td {
  background-color: rgba(var(--v-theme-primary), 0.08);
}

.task-table .v-table__wrapper > table {
  width: max-content;
  min-width: 100%;
}
</style>