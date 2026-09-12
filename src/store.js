import { defineStore } from "pinia";
import { api } from "./api";

const COLUMNS_KEY = "tt.columns";

function defaultColumns() {
  return [
    { key: "time", label: "Время", visible: true, width: '100px' },
    { key: "status", label: "Статус", visible: true, width: '' },
    { key: "actions", label: "Действия", visible: true, width: '' },
    { key: "mode", label: "Режим", visible: true, width: '' },
    { key: "order", label: "Заявка", visible: true, width: '' },
    { key: "client", label: "Клиент", visible: true, width: '' },
    { key: "tags", label: "Тег", visible: true, width: '' },
    { key: "start", label: "Начало", visible: true, width: '180px' },
    { key: "end", label: "Завершение", visible: true, width: '180px' },

    { key: "user", label: "Пользователь", visible: true, width: '' },
    { key: "comment", label: "Комментарий", visible: true, width: '' },
    { key: "taskId", label: "ID", visible: true, width: '' },
  ]
}

function loadColumns() {
  const defs = defaultColumns();
  const byKey = new Map(defs.map((d) => [d.key, d]));
  let stored = null;
  try {
    stored = JSON.parse(localStorage.getItem(COLUMNS_KEY) || "null");
  } catch {
    stored = null;
  }
  if (!Array.isArray(stored)) return defs;
  const out = [];
  const used = new Set();
  for (const s of stored) {
    // Миграция: в старых версиях key "status" означал «Режим» (running/пауза),
    // новая колонка «Статус» получает отдельный key "status".
    let key = s?.key === "status" ? "mode" : s?.key;
    // "tag" переименован в "tags" (совпадение с полем строки для сортировки).
    if (key === "tag") key = "tags";
    const d = byKey.get(key);
    if (d && !used.has(key)) {
      used.add(key);
      out.push({ ...d, visible: s.visible !== false });
    }
  }
  for (const d of defs) {
    if (!used.has(d.key)) {
      used.add(d.key);
      out.push(d);
    }
  }
  return out;
}

export const useAppStore = defineStore("app", {
  state: () => ({
    ready: false,
    loading: true,
    error: "",
    settings: { username: "", grouping: "none" },
    users: [],
    tags: [],
    clients: [],
    statuses: [],
    rows: [],
    totals: { count: 0, totalSeconds: 0, timeLabel: "00:00:00", byTag: [] },
    filter: {
      dateFrom: "",
      dateTo: "",
      tags: [],
      client: "",
      user: "",
      search: "",
    },
    columns: loadColumns(),
  }),

  getters: {
    visibleColumns: (state) => state.columns.filter((c) => c.visible)
  },

  actions: {
    async init() {
      await this.refresh();
      this.loading = false;
    },

    async refresh() {
      try {
        const s = await api.getState();
        this.settings = s.settings;
        this.users = s.users || [];
        this.tags = s.tags || [];
        this.clients = s.clients || [];
        this.statuses = s.statuses || [];
        this.error = "";
      } catch (e) {
        this.error = String(e);
      }
    },

    async refreshQuery() {
      try {
        const q = await api.query(this.filter);
        this.rows = q.rows.map(it => {
          return {...it, actions: null, group: null}
        })
        this.totals = q.totals
        this.error = ""
      } catch (e) {
        this.error = String(e);
      }
    },

    async applySettings(username, grouping) {
      await api.setSettings(username, grouping);
      this.settings.username = username;
      this.settings.grouping = grouping;
      await this.refresh();
      await this.refreshQuery();
    },

    async addTask(draft) {
      const created = await api.createTask(draft);
      await this.refresh();
      await this.refreshQuery();
      return created;
    },

    async updateTask(taskId, draft) {
      await api.updateTask({ taskId, ...draft });
      await this.refresh();
      await this.refreshQuery();
    },

    async updateTaskDates(taskId, start, end) {
      await api.setTaskDates(taskId, start, end);
      await this.refresh();
      await this.refreshQuery();
    },

    async updateTaskIntervals(taskId, lines) {
      await api.setTaskIntervals(taskId, lines);
      await this.refresh();
      await this.refreshQuery();
    },

    async updateTaskStatus(taskId, status) {
      await api.setTaskStatus(taskId, status);
      await this.refresh();
      await this.refreshQuery();
    },

    async startTask(id) {
      await api.startTask(id);
      await this.refreshQuery();
    },

    async pauseTask(id) {
      await api.pauseTask(id);
      await this.refreshQuery();
    },

    async resumeTask(id) {
      await api.resumeTask(id);
      await this.refreshQuery();
    },

    async completeTask(id) {
      await api.completeTask(id);
      await this.refreshQuery();
    },

    async removeTasks(ids) {
      await api.removeTasks(ids);
      await this.refresh();
      await this.refreshQuery();
    },

    async saveEntity(kind, payload) {
      // kind: user | tag | client | status ; payload: {action, ...}
      if (kind === "user") {
        if (payload.action === "add") await api.addUser(payload.name);
        else if (payload.action === "rename") await api.renameUser(payload.old, payload.name);
        else if (payload.action === "remove") await api.removeUser(payload.name);
        else if (payload.action === "move") await api.moveUser(payload.from, payload.to);
        else if (payload.action === "clear") await api.clearUsers();
      } else if (kind === "tag") {
        if (payload.action === "add") await api.addTag(payload.name);
        else if (payload.action === "rename") await api.renameTag(payload.old, payload.name);
        else if (payload.action === "remove") await api.removeTag(payload.id);
        else if (payload.action === "move") await api.moveTag(payload.from, payload.to);
        else if (payload.action === "clear") await api.clearTags();
      } else if (kind === "client") {
        if (payload.action === "add") await api.addClient(payload.name);
        else if (payload.action === "rename") await api.renameClient(payload.old, payload.name);
        else if (payload.action === "remove") await api.removeClient(payload.id);
        else if (payload.action === "move") await api.moveClient(payload.from, payload.to);
        else if (payload.action === "clear") await api.clearClients();
      } else if (kind === "status") {
        if (payload.action === "add") await api.addStatus(payload.name, payload.color || "default");
        else if (payload.action === "rename") await api.renameStatus(payload.old, payload.name);
        else if (payload.action === "remove") await api.removeStatus(payload.id);
        else if (payload.action === "color") await api.setStatusColor(payload.id, payload.color);
        else if (payload.action === "move") await api.moveStatus(payload.from, payload.to);
        else if (payload.action === "clear") await api.clearStatuses();
      }
      await this.refresh();
      await this.refreshQuery();
    },

    persistColumns() {
      try {
        localStorage.setItem(COLUMNS_KEY, JSON.stringify(this.columns));
      } catch {
        /* ignore */
      }
    },

    /** Вернуть колонки к порядку по умолчанию (видимость сохраняется). */
    resetColumns() {
      const byKey = new Map(this.columns.map((c) => [c.key, c]));
      const out = [];
      const used = new Set();
      for (const d of defaultColumns()) {
        used.add(d.key);
        out.push(byKey.get(d.key) || { ...d });
      }
      for (const c of this.columns) {
        if (!used.has(c.key)) {
          used.add(c.key);
          out.push(c);
        }
      }
      this.columns = out;
      this.persistColumns();
    },

    moveColumn(from, to) {
      const arr = [...this.columns];
      if (from < 0 || from >= arr.length || to < 0 || to >= arr.length || from === to) return;
      const [col] = arr.splice(from, 1);
      arr.splice(to, 0, col);
      this.columns = arr;
      this.persistColumns();
    },

    setColumnVisible(key, visible) {
      const col = this.columns.find((c) => c.key === key);
      if (col) col.visible = !!visible;
      this.persistColumns();
    },
  },
});