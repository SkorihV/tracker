import { defineStore } from "pinia"
import { api, onApiError } from "../api"
import { useColumnsStore } from "./columns"
import { useDictionariesStore } from "./dictionaries"
import { useSettingsStore } from "./settings"
import { useTasksStore } from "./tasks"

/** Корневой стор: загрузка данных, ошибки и обновление остальных сторов. */
export const useAppStore = defineStore("app", {
  state: () => ({
    showSettings: false,
    loading: true,
    error: "",
  }),

  actions: {
    async init() {
      onApiError((message) => {
        this.error = message
      })
      await this.sync()
      this.loading = false
    },

    /** Настройки, справочники и колонки с бэкенда. */
    async refresh() {
      try {
        const s = await api.getState()
        useSettingsStore().hydrate(s.settings)
        useColumnsStore().hydrate(s.settings?.columns)
        useDictionariesStore().hydrate(s)
      } catch {
        /* текст ошибки уже показан через onApiError */
      }
    },

    /** Справочники и таблица за один заход (параллельно). */
    async sync() {
      await Promise.all([this.refresh(), this.refreshQuery()])
    },

    /** Обновить только таблицу задач (вызывается и по таймеру). */
    async refreshQuery() {
      try {
        await useTasksStore().refreshQuery()
      } catch {
        /* текст ошибки уже показан через onApiError */
      }
    },
  },
})
