import { defineStore } from "pinia"
import { api } from "../api"
import { useAppStore } from "./app"
import vuetify from "../plugins/vuetify"
import { readCookie, writeCookie } from "../utils/cookies"

export const DEFAULT_ACCENT_COLOR = "#4caf50"
export const DEFAULT_OUR_CAR_COLOR = "#2196f3"
export const DEFAULT_TABLE_FONT_FAMILY = "Roboto"
export const DEFAULT_TABLE_FONT_SIZE = 14

const HEX_RE = /^#[0-9a-fA-F]{6}$/

const hexOr = (value, fallback) => (HEX_RE.test(value || "") ? value : fallback)
const validFontSize = (n) => Number.isInteger(n) && n >= 8 && n <= 40

function readFontFamily() {
  const v = readCookie("tt_font").trim()
  return v || DEFAULT_TABLE_FONT_FAMILY
}

function readFontSize() {
  const n = Number(readCookie("tt_fontsize"))
  return validFontSize(n) ? n : DEFAULT_TABLE_FONT_SIZE
}

function readTheme() {
  return readCookie("tt_theme") === "dark" ? "dark" : "light"
}

function applyAccentToTheme(color) {
  const themes = vuetify.theme.themes.value
  for (const name of ["light", "dark"]) {
    if (themes[name]?.colors) themes[name].colors.primary = color
  }
}

function applyTheme(name) {
  vuetify.theme.change(name)
}

/** Настройки пользователя: часть лежит в файле данных, часть (шрифт, тема) — в cookie. */
export const useSettingsStore = defineStore("settings", {
  state: () => ({
    settings: {
      username: "",
      grouping: "none",
      accentColor: DEFAULT_ACCENT_COLOR,
      ourCarColor: DEFAULT_OUR_CAR_COLOR,
      fontFamily: readFontFamily(),
      fontSize: readFontSize(),
      backupDir: "",
      theme: readTheme(),
      minimizeToTray: false,
      reminderHours: 0,
    },
  }),

  actions: {
    /** Принять настройки, пришедшие с бэкенда (`get_state().settings`). */
    hydrate(s = {}) {
      this.settings = {
        username: s.username || "",
        grouping: s.grouping || "none",
        accentColor: hexOr(s.accent_color, DEFAULT_ACCENT_COLOR),
        ourCarColor: hexOr(s.our_car_color, DEFAULT_OUR_CAR_COLOR),
        fontFamily: readFontFamily(),
        fontSize: readFontSize(),
        backupDir: s.backup_dir || "",
        theme: readTheme(),
        minimizeToTray: !!s.minimize_to_tray,
        reminderHours: s.reminder_hours || 0,
      }
      applyAccentToTheme(this.settings.accentColor)
      applyTheme(this.settings.theme)
    },

    async applySettings(username, grouping) {
      await api.setSettings(username, grouping, this.settings.accentColor)
      this.settings.username = username
      this.settings.grouping = grouping
      await useAppStore().sync()
    },

    async setAccentColor(color) {
      const c = hexOr(color, DEFAULT_ACCENT_COLOR)
      await api.setSettings(this.settings.username, this.settings.grouping, c)
      this.settings.accentColor = c
      applyAccentToTheme(c)
    },

    async setOurCarColor(color) {
      const c = hexOr(color, DEFAULT_OUR_CAR_COLOR)
      await api.setOurCarColor(c)
      this.settings.ourCarColor = c
    },

    /** Вид и размер шрифта таблицы (cookie, без обращения к бэкенду). */
    setTableFont(family, size) {
      const f = String(family || "").trim() || DEFAULT_TABLE_FONT_FAMILY
      const n = Number(size)
      const sz = validFontSize(n) ? n : DEFAULT_TABLE_FONT_SIZE
      writeCookie("tt_font", f)
      writeCookie("tt_fontsize", String(sz))
      this.settings.fontFamily = f
      this.settings.fontSize = sz
    },

    setTheme(theme) {
      const t = theme === "dark" ? "dark" : "light"
      this.settings.theme = t
      writeCookie("tt_theme", t)
      applyTheme(t)
    },

    /** Сворачивание в трей и порог напоминания о забытом таймере (часы, 0 = выкл.). */
    async setTrayPrefs(minimizeToTray, reminderHours) {
      const hours = Math.max(0, Math.min(72, Math.floor(Number(reminderHours) || 0)))
      await api.setTrayPrefs(!!minimizeToTray, hours)
      this.settings.minimizeToTray = !!minimizeToTray
      this.settings.reminderHours = hours
    },

    /** Пользовательский каталог бэкапов ("" = по умолчанию). */
    async setBackupsDir(dir) {
      const d = String(dir || "").trim()
      await api.setBackupsDir(d)
      this.settings.backupDir = d
    },

    async getDefaultBackupsDir() {
      return (await api.getDefaultBackupsDir()) || ""
    },

    /** Фактический каталог бэкапов (пользовательский либо по умолчанию). */
    async effectiveBackupsDir() {
      return this.settings.backupDir || this.getDefaultBackupsDir()
    },

    async getDbDir() {
      return (await api.getDbDir()) || ""
    },

    openInExplorer(dir) {
      return api.openInExplorer(dir)
    },
  },
})
