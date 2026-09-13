import { defineStore } from "pinia"
import { api } from "./api"
import vuetify from "./plugins/vuetify"
import { today } from "./reportFilter"

export const DEFAULT_ACCENT_COLOR = "#4caf50"
export const DEFAULT_OUR_CAR_COLOR = "#2196f3"
export const DEFAULT_TABLE_FONT_FAMILY = "Roboto"
export const DEFAULT_TABLE_FONT_SIZE = 14

const HEX_RE = /^#[0-9a-fA-F]{6}$/

function applyAccentToTheme(color) {
  const c = HEX_RE.test(color || "") ? color : DEFAULT_ACCENT_COLOR
  try {
    const light = vuetify?.theme?.themes?.value?.light
    if (light && light.colors) light.colors.primary = c
  } catch {
    /* ignore */
  }
}

function defaultColumns() {
  return [
    { key: "time", label: "Время", visible: true, width: '100px' },
    { key: "status", label: "Статус", visible: true, width: '' },
    { key: "ourCar", label: "Наша машина", visible: true, width: '' },
    { key: "actions", label: "Действия", visible: true, width: '' },
    { key: "mode", label: "Режим", visible: true, width: '' },
    { key: "order", label: "Заявка", visible: true, width: '' },
    { key: "client", label: "Клиент", visible: true, width: '' },
    { key: "tags", label: "Тег", visible: true, width: '' },
    { key: "start", label: "Начало", visible: true, width: '180px' },
    { key: "end", label: "Завершение", visible: true, width: '180px' },

    { key: "user", label: "Пользователь", visible: true, width: '' },
    { key: "comment", label: "Комментарий", visible: true, width: '' },
    { key: "taskId", label: "ID", visible: false, width: '' },
  ]
}

function loadColumns() {
  return defaultColumns()
}

// Порядок колонок, хранящийся на бэкенде. Данные приходят уже в нужном
// порядке и с корректными ключами — просто дополняем метаданными defs.
function mergeColumnsFromBackend(prefs) {
  const defs = defaultColumns()
  const byKey = new Map(defs.map((d) => [d.key, d]))
  if (!Array.isArray(prefs) || prefs.length === 0) return defs
  const out = []
  const used = new Set()
  for (const s of prefs) {
    if (typeof s.key !== "string") continue
    const d = byKey.get(s.key)
    if (d && !used.has(s.key)) {
      used.add(s.key)
      out.push({ ...d, visible: s.visible !== false })
    }
  }
  for (const d of defs) {
    if (!used.has(d.key)) {
      used.add(d.key)
      out.push(d)
    }
  }
  return out
}

export const useAppStore = defineStore("app", {
  state: () => ({
    showSettings: false,
    ready: false,
    loading: true,
    error: "",
    settings: { username: "", grouping: "none", accentColor: DEFAULT_ACCENT_COLOR, ourCarColor: DEFAULT_OUR_CAR_COLOR, fontFamily: DEFAULT_TABLE_FONT_FAMILY, fontSize: DEFAULT_TABLE_FONT_SIZE, backupDir: "" },
    users: [],
    tags: [],
    clients: [],
    statuses: [],
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
    columns: loadColumns(),
  }),

  getters: {
    visibleColumns: (state) => state.columns.filter((c) => c.visible)
  },

  actions: {
    async init() {
      await this.refresh()
      this.loading = false
    },

    async refresh() {
      try {
        const s = await api.getState()
        this.settings = {
          username: s.settings?.username || "",
          grouping: s.settings?.grouping || "none",
          accentColor: HEX_RE.test(s.settings?.accent_color || "") ? s.settings.accent_color : DEFAULT_ACCENT_COLOR,
          ourCarColor: HEX_RE.test(s.settings?.our_car_color || "") ? s.settings.our_car_color : DEFAULT_OUR_CAR_COLOR,
          fontFamily: s.settings?.font_family && s.settings.font_family.trim() !== "" ? s.settings.font_family : DEFAULT_TABLE_FONT_FAMILY,
          fontSize: s.settings?.font_size >= 8 && s.settings.font_size <= 40 ? s.settings.font_size : DEFAULT_TABLE_FONT_SIZE,
          backupDir: s.settings?.backup_dir || "",
          columns: s.settings?.columns || [],
        }
        this.columns = mergeColumnsFromBackend(s.settings?.columns)
        this.users = s.users || []
        this.tags = s.tags || []
        this.clients = s.clients || []
        this.statuses = s.statuses || []
        this.error = ""
        applyAccentToTheme(this.settings.accentColor)
      } catch (e) {
        this.error = String(e)
      }
    },

    async refreshQuery() {
      try {
        const q = await api.query(this.filter)
        this.rows = q.rows.map(it => {
          return {...it, actions: null, group: null}
        })
        this.totals = q.totals
        this.error = ""
      } catch (e) {
        this.error = String(e)
      }
    },

    async applySettings(username, grouping) {
      await api.setSettings(username, grouping, this.settings.accentColor)
      this.settings.username = username
      this.settings.grouping = grouping
      await this.refresh()
      await this.refreshQuery()
    },

    /** Изменить акцентный цвет кнопок и заголовков. */
    async setAccentColor(color) {
      const c = HEX_RE.test(color || "") ? color : DEFAULT_ACCENT_COLOR
      await api.setSettings(this.settings.username, this.settings.grouping, c)
      this.settings.accentColor = c
      applyAccentToTheme(c)
      await this.refreshQuery()
    },

    /** Изменить цвет ячейки «Наша машина». */
    async setOurCarColor(color) {
      const c = HEX_RE.test(color || "") ? color : DEFAULT_OUR_CAR_COLOR
      await api.setOurCarColor(c)
      this.settings.ourCarColor = c
      await this.refreshQuery()
    },

    /** Изменить вид и размер шрифта таблицы задач. */
    async setTableFont(family, size) {
      const f = String(family || "").trim() || DEFAULT_TABLE_FONT_FAMILY
      const s = Number(size)
      const sz = Number.isInteger(s) && s >= 8 && s <= 40 ? s : DEFAULT_TABLE_FONT_SIZE
      await api.setTableFont(f, sz)
      this.settings.fontFamily = f
      this.settings.fontSize = sz
      await this.refreshQuery()
    },

    async addTask(draft) {
      const created = await api.createTask(draft)
      await this.refresh()
      await this.refreshQuery()
      return created
    },

    async updateTask(taskId, draft) {
      await api.updateTask({ taskId, ...draft })
      await this.refresh()
      await this.refreshQuery()
    },

    async updateTaskDates(taskId, start, end) {
      await api.setTaskDates(taskId, start, end)
      await this.refresh()
      await this.refreshQuery()
    },

    async updateTaskIntervals(taskId, lines) {
      await api.setTaskIntervals(taskId, lines)
      await this.refresh()
      await this.refreshQuery()
    },

    async updateTaskStatus(taskId, status) {
      await api.setTaskStatus(taskId, status)
      await this.refresh()
      await this.refreshQuery()
    },

    async startTask(id) {
      await api.startTask(id)
      await this.refreshQuery()
    },

    async pauseTask(id) {
      await api.pauseTask(id)
      await this.refreshQuery()
    },

    async resumeTask(id) {
      await api.resumeTask(id)
      await this.refreshQuery()
    },

    async completeTask(id) {
      await api.completeTask(id)
      await this.refreshQuery()
    },

    async removeTasks(ids) {
      await api.removeTasks(ids)
      await this.refresh()
      await this.refreshQuery()
    },

    async saveEntity(kind, payload) {
      // kind: user | tag | client | status ; payload: {action, ...}
      if (kind === "user") {
        if (payload.action === "add") await api.addUser(payload.name)
        else if (payload.action === "rename") await api.renameUser(payload.old, payload.name)
        else if (payload.action === "remove") await api.removeUser(payload.name)
        else if (payload.action === "move") await api.moveUser(payload.from, payload.to)
        else if (payload.action === "clear") await api.clearUsers()
      } else if (kind === "tag") {
        if (payload.action === "add") await api.addTag(payload.name)
        else if (payload.action === "rename") await api.renameTag(payload.old, payload.name)
        else if (payload.action === "remove") await api.removeTag(payload.id)
        else if (payload.action === "move") await api.moveTag(payload.from, payload.to)
        else if (payload.action === "clear") await api.clearTags()
      } else if (kind === "client") {
        if (payload.action === "add") await api.addClient(payload.name)
        else if (payload.action === "rename") await api.renameClient(payload.old, payload.name)
        else if (payload.action === "remove") await api.removeClient(payload.id)
        else if (payload.action === "move") await api.moveClient(payload.from, payload.to)
        else if (payload.action === "clear") await api.clearClients()
      } else if (kind === "status") {
        if (payload.action === "add") await api.addStatus(payload.name, payload.color || "default")
        else if (payload.action === "rename") await api.renameStatus(payload.old, payload.name)
        else if (payload.action === "remove") await api.removeStatus(payload.id)
        else if (payload.action === "color") await api.setStatusColor(payload.id, payload.color)
        else if (payload.action === "move") await api.moveStatus(payload.from, payload.to)
        else if (payload.action === "clear") await api.clearStatuses()
      }
      await this.refresh()
      await this.refreshQuery()
    },

    async persistColumns() {
      const list = this.columns.map((c) => ({ key: c.key, visible: c.visible }))
      try {
        await api.setColumns(list)
      } catch {
        /* ignore */
      }
    },

    /** Вернуть колонки к порядку по умолчанию (видимость сохраняется). */
    async resetColumns() {
      const byKey = new Map(this.columns.map((c) => [c.key, c]))
      const out = []
      const used = new Set()
      for (const d of defaultColumns()) {
        used.add(d.key)
        out.push(byKey.get(d.key) || { ...d })
      }
      for (const c of this.columns) {
        if (!used.has(c.key)) {
          used.add(c.key)
          out.push(c)
        }
      }
      this.columns = out
      await this.persistColumns()
    },

    async moveColumn(from, to) {
      const arr = [...this.columns]
      if (from < 0 || from >= arr.length || to < 0 || to >= arr.length || from === to) return
      const [col] = arr.splice(from, 1)
      arr.splice(to, 0, col)
      this.columns = arr
      await this.persistColumns()
    },

    async setColumnVisible(key, visible) {
      const col = this.columns.find((c) => c.key === key)
      if (col) col.visible = !!visible
      await this.persistColumns()
    },

    /** Установить пользовательский каталог бэкапов ("" = по умолчанию). */
    async setBackupsDir(dir) {
      const d = String(dir || "").trim()
      await api.setBackupsDir(d)
      this.settings.backupDir = d
      await this.refresh()
    },

    /** Путь каталога бэкапов по умолчанию. */
    async getDefaultBackupsDir() {
      return (await api.getDefaultBackupsDir()) || ""
    },

    /** Фактический каталог бэкапов (пользовательский либо по умолчанию). */
    async effectiveBackupsDir() {
      if (this.settings.backupDir) return this.settings.backupDir
      return this.getDefaultBackupsDir()
    },
  },
})