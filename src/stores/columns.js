import { defineStore } from "pinia"
import { api } from "../api"
import { mergeColumns, resetColumnOrder } from "../columns"

/** Порядок и видимость колонок таблицы (хранятся на бэкенде). */
export const useColumnsStore = defineStore("columns", {
  state: () => ({
    columns: mergeColumns([]),
  }),

  getters: {
    visibleColumns: (state) => state.columns.filter((c) => c.visible),
  },

  actions: {
    hydrate(prefs) {
      this.columns = mergeColumns(prefs)
    },

    persist() {
      return api.setColumns(this.columns.map((c) => ({ key: c.key, visible: c.visible })))
    },

    async reset() {
      this.columns = resetColumnOrder(this.columns)
      await this.persist()
    },

    async move(from, to) {
      const arr = [...this.columns]
      if (from < 0 || from >= arr.length || to < 0 || to >= arr.length || from === to) return
      const [col] = arr.splice(from, 1)
      arr.splice(to, 0, col)
      this.columns = arr
      await this.persist()
    },

    async setVisible(key, visible) {
      const col = this.columns.find((c) => c.key === key)
      if (!col) return
      col.visible = !!visible
      await this.persist()
    },
  },
})
