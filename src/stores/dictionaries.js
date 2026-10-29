import { defineStore } from "pinia"
import { api } from "../api"
import { useAppStore } from "./app"

/** Справочники: пользователи, теги, клиенты, статусы. */
export const useDictionariesStore = defineStore("dictionaries", {
  state: () => ({
    users: [],
    tags: [],
    clients: [],
    statuses: [],
  }),

  actions: {
    hydrate(s = {}) {
      this.users = s.users || []
      this.tags = s.tags || []
      this.clients = s.clients || []
      this.statuses = s.statuses || []
    },

    /**
     * Правка справочника. kind: user | tag | client | status;
     * payload: { action: add | rename | remove | move | clear | color, ... }.
     */
    async saveEntity(kind, payload) {
      await api.editDictionary(kind, payload)
      await useAppStore().sync()
    },
  },
})
