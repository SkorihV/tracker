import { defineStore } from "pinia"
import { api } from "../api"
import { today } from "../reportFilter"
import { useAppStore } from "./app"

/** Задачи на экране: фильтр, строки, итоги и действия над задачами. */
export const useTasksStore = defineStore("tasks", {
  state: () => ({
    rows: [],
    totals: { count: 0, totalSeconds: 0, timeLabel: "00:00:00", byTag: [] },
    filter: {
      dateFrom: today(),
      dateTo: today(),
      tags: [],
      client: "",
      user: "",
      search: "",
    },
  }),

  getters: {
    hasRunning: (state) => state.rows.some((r) => r.mode === "running"),
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

    removeTasks(ids) {
      return this.mutate(() => api.removeTasks(ids))
    },
  },
})
