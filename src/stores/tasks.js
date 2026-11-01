import { defineStore } from "pinia"
import { api } from "../api"
import { today } from "../reportFilter"
import { MAX_PINNED, readPinnedIds, savePinnedIds } from "../utils/cookies"
import { useAppStore } from "./app"

/** Задачи на экране: фильтр, строки, итоги и действия над задачами. */
export const useTasksStore = defineStore("tasks", {
  state: () => ({
    rows: [],
    totals: { count: 0, totalSeconds: 0, timeLabel: "00:00:00", actualSeconds: 0, actualLabel: "00:00:00", byTag: [] },
    filter: {
      dateFrom: today(),
      dateTo: today(),
      tags: [],
      client: "",
      user: "",
      search: "",
    },
    // Закреплённые задачи хранятся в cookie, в данные и отчёты не попадают.
    pinnedIds: readPinnedIds(),
  }),

  getters: {
    hasRunning: (state) => state.rows.some((r) => r.mode === "running"),

    /** Закреплённые строки в порядке закрепления; только те, что попали в текущий фильтр. */
    pinnedRows: (state) => {
      const byId = new Map(state.rows.map((r) => [r.taskId, r]))
      return state.pinnedIds.map((id) => byId.get(id)).filter(Boolean)
    },

    /** Остальные строки (то, что сортируется и группируется). */
    unpinnedRows: (state) => {
      const pinned = new Set(state.pinnedIds)
      return state.rows.filter((r) => !pinned.has(r.taskId))
    },
  },

  actions: {
    async refreshQuery() {
      const q = await api.query(this.filter)
      this.rows = q.rows.map((it) => ({ ...it, actions: null, group: null }))
      this.totals = q.totals
    },

    /** Правка, после которой могли измениться справочники (новый тег, клиент, статус…). */
    async mutateWithDictionaries(fn) {
      const result = await fn()
      await useAppStore().sync()
      return result
    },

    /** Правка, которая меняет только строки таблицы. */
    async mutate(fn) {
      const result = await fn()
      await this.refreshQuery()
      return result
    },

    addTask(draft) {
      return this.mutateWithDictionaries(() => api.createTask(draft))
    },

    updateTask(taskId, draft) {
      return this.mutateWithDictionaries(() => api.updateTask(taskId, draft))
    },

    updateTaskStatus(taskId, status) {
      return this.mutateWithDictionaries(() => api.setTaskStatus(taskId, status))
    },

    updateTaskDates(taskId, start, end) {
      return this.mutate(() => api.setTaskDates(taskId, start, end))
    },

    updateTaskIntervals(taskId, lines) {
      return this.mutate(() => api.setTaskIntervals(taskId, lines))
    },

    /** action: start | pause | resume | complete */
    taskAction(taskId, action) {
      return this.mutate(() => api.taskAction(taskId, action))
    },

    async removeTasks(ids) {
      await this.mutate(() => api.removeTasks(ids))
      this.unpin(ids)
    },

    togglePin(taskId) {
      if (this.pinnedIds.includes(taskId)) {
        this.unpin([taskId])
      } else if (this.pinnedIds.length >= MAX_PINNED) {
        useAppStore().error = `Можно закрепить не более ${MAX_PINNED} задач`
      } else {
        this.pinnedIds = [...this.pinnedIds, taskId]
        savePinnedIds(this.pinnedIds)
      }
    },

    /** Сбросить все закреплённые задачи (в том числе не попавшие в текущий фильтр). */
    clearPins() {
      if (!this.pinnedIds.length) return
      this.pinnedIds = []
      savePinnedIds(this.pinnedIds)
    },

    unpin(ids) {
      const drop = new Set(ids)
      if (!this.pinnedIds.some((id) => drop.has(id))) return
      this.pinnedIds = this.pinnedIds.filter((id) => !drop.has(id))
      savePinnedIds(this.pinnedIds)
    },
  },
})
