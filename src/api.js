import { invoke } from "@tauri-apps/api/core"

let errorHandler = null

/** Зарегистрировать обработчик, который получает текст любой ошибки команды. */
export function onApiError(handler) {
  errorHandler = handler
}

async function call(cmd, args) {
  try {
    return await invoke(cmd, args)
  } catch (e) {
    console.error(`[${cmd}]`, e)
    errorHandler?.(String(e))
    throw e
  }
}

function cleanFilter(f) {
  const out = {}
  for (const [k, v] of Object.entries(f)) out[k] = v ?? ""
  return out
}

/** Оболочка над командами Tauri. Типы аргументов — в src-tauri/src/commands.rs. */
export const api = {
  /** Настройки и справочники: { settings, users, tags, clients, statuses }. */
  getState: () => call("get_state"),

  /** Задачи по фильтру: { rows, totals }. Групповые строки отбрасываются. */
  query: async (filter) => {
    const res = await call("query", { filter: cleanFilter(filter) })
    return {
      ...res,
      rows: (res.rows || []).filter((r) => r.kind !== "group").map(({ kind: _kind, ...row }) => row),
    }
  },

  reportPreview: (filter) => call("report_preview", { filter: cleanFilter(filter) }),
  statsPreview: (filter) => call("stats_preview", { filter: cleanFilter(filter) }),

  /** Сформировать файл отчёта (txt | md | csv | xlsx). Без path — в reports/. Возвращает путь. */
  createReport: (format, filter, dateFrom, dateTo, path = "") =>
    call("create_report", { format, filter: cleanFilter(filter), dateFrom, dateTo, path }),
  createStats: (format, filter, dateFrom, dateTo, path = "") =>
    call("create_stats", { format, filter: cleanFilter(filter), dateFrom, dateTo, path }),

  exportJson: (path, opts) =>
    call("export_json", {
      path,
      exportTasks: opts.tasks,
      exportTags: opts.tags,
      exportClients: opts.clients,
      exportUsers: opts.users,
      exportStatuses: opts.statuses,
    }),
  importJson: (path, opts) =>
    call("import_json", {
      path,
      importTasks: opts.tasks,
      importTags: opts.tags,
      importClients: opts.clients,
      importUsers: opts.users,
      importStatuses: opts.statuses,
    }),

  /** draft: { user, orders, tags, client, comment, customStatus, ourCar }. */
  createTask: (draft) => call("create_task", { draft }),
  updateTask: (taskId, draft) => call("update_task", { taskId, draft }),
  setTaskStatus: (taskId, status) => call("set_task_status", { taskId, status }),
  setTaskDates: (taskId, start, end) => call("set_task_dates", { taskId, start, end }),
  setTaskIntervals: (taskId, lines) => call("set_task_intervals", { taskId, lines }),
  /** action: start | pause | resume | complete */
  taskAction: (taskId, action) => call("task_action", { taskId, action }),
  removeTasks: (ids) => call("remove_tasks", { ids }),

  /**
   * Правка справочника. kind: user | tag | client | status;
   * action: { action: "add" | "rename" | "remove" | "move" | "clear" | "color", ... }.
   */
  editDictionary: (kind, action) => call("edit_dictionary", { kind, action }),

  setSettings: (username, grouping, accentColor) => call("set_settings", { username, grouping, accentColor }),
  setOurCarColor: (color) => call("set_our_car_color", { color }),
  setColumns: (columns) => call("set_columns", { columns }),
  setTrayPrefs: (minimizeToTray, reminderHours) => call("set_tray_prefs", { minimizeToTray, reminderHours }),
  setBackupsDir: (dir) => call("set_backups_dir", { dir }),

  getDefaultBackupsDir: () => call("default_backups_dir"),
  getDbDir: () => call("db_dir"),
  openInExplorer: (dir) => call("open_in_explorer", { path: dir }),
}

export function fmtTd(seconds) {
  const t = Math.max(0, Math.floor(seconds || 0))
  const h = Math.floor(t / 3600)
  const m = Math.floor((t % 3600) / 60)
  const s = t % 60
  return [h, m, s].map((x) => String(x).padStart(2, "0")).join(":")
}
