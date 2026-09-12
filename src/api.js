import { invoke } from "@tauri-apps/api/core";

async function call(cmd, args) {
  try {
    return await invoke(cmd, args);
  } catch (e) {
    console.error(`[${cmd}]`, e);
    throw e;
  }
}

function cleanFilter(f) {
  const out = {};
  for (const [k, v] of Object.entries(f)) out[k] = v ?? "";
  return out;
}

export const api = {
  /** Получить всё состояние приложения: settings, users, tags, clients, statuses. */
  getState: () => call("get_state"),

  /** Выполнить запрос задач по фильтру. Возвращает { rows, totals }. Строки — только заявки (group-строки и поле kind отбрасываются). */
  query: async (filter) => {
    const res = await call("query", { filter: cleanFilter(filter) });
    return {
      ...res,
      rows: (res.rows || [])
        .filter((r) => r.kind !== "group")
        .map(({ kind, ...row }) => row),
    };
  },

  /** Предпросмотр отчёта (данные на экране, без записи файла). */
  reportPreview: (filter) => call("report_preview", { filter: cleanFilter(filter) }),

  /** Сформировать файл отчёта. Возвращает путь к созданному файлу. */
  createReport: (format, filter, dateFrom, dateTo) =>
    call("create_report", { format, filter: cleanFilter(filter), dateFrom, dateTo }),

  /** Предпросмотр статистики (на экран, без записи файла). */
  statsPreview: (filter) => call("stats_preview", { filter: cleanFilter(filter) }),

  /** Сформировать файл статистики. Возвращает путь к созданному файлу. */
  createStats: (format, filter, dateFrom, dateTo) =>
    call("create_stats", { format, filter: cleanFilter(filter), dateFrom, dateTo }),

  /** Экспортировать данные в JSON-файл по указанному пути. */
  exportJson: (path, opts) =>
    call("export_json", {
      path,
      exportTasks: opts.tasks,
      exportTags: opts.tags,
      exportClients: opts.clients,
      exportUsers: opts.users,
    }),

  /** Импортировать данные из JSON-файла по указанному пути. */
  importJson: (path, opts) =>
    call("import_json", {
      path,
      importTasks: opts.tasks,
      importTags: opts.tags,
      importClients: opts.clients,
      importUsers: opts.users,
    }),

  /** Создать задачу из черновика { user, order, tags, client, comment, customStatus }. Возвращает AppTask. */
  createTask: (d) => call("create_task", d),

  /** Обновить задачу из черновика (те же поля, что у createTask). */
  updateTask: (d) => call("update_task", d),

  /** Установить пользовательский статус задачи (справочник статусов). */
  setTaskStatus: (taskId, status) => call("set_task_status", { taskId, status }),

  /** Переопределить даты начала/завершения задачи. */
  setTaskDates: (taskId, start, end) => call("set_task_dates", { taskId, start, end }),

  /** Переопределить список интервалов работы задачи (строки "HH:MM-HH:MM"). */
  setTaskIntervals: (taskId, lines) => call("set_task_intervals", { taskId, lines }),

  /** Запустить задачу (старт замера времени). */
  startTask: (taskId) => call("start_task", { taskId }),

  /** Поставить задачу на паузу. */
  pauseTask: (taskId) => call("pause_task", { taskId }),

  /** Возобновить задачу после паузы. */
  resumeTask: (taskId) => call("resume_task", { taskId }),

  /** Завершить задачу (фиксация итогового времени). */
  completeTask: (taskId) => call("complete_task", { taskId }),

  /** Удалить задачи по ID. */
  removeTasks: (ids) => call("remove_tasks", { ids }),

  /** Сохранить настройки: имя пользователя и режим группировки. */
  setSettings: (username, grouping) => call("set_settings", { username, grouping }),

  /** Добавить пользователя в справочник. */
  addUser: (name) => call("add_user", { name }),

  /** Удалить пользователя из справочника. */
  removeUser: (name) => call("remove_user", { name }),

  /** Удалить всех пользователей из справочника. */
  clearUsers: () => call("clear_users"),

  /** Переименовать пользователя. */
  renameUser: (old, newName) => call("rename_user", { old, new: newName }),

  /** Переместить пользователя в списке (from → to). */
  moveUser: (from, to) => call("move_user", { from, to }),

  /** Добавить тег в справочник. */
  addTag: (name) => call("add_tag", { name }),

  /** Удалить тег по ID (с каскадным удалением из задач). */
  removeTag: (id) => call("remove_tag", { id }),

  /** Удалить все теги из справочника. */
  clearTags: () => call("clear_tags"),

  /** Переименовать тег (обновляется во всех задачах). */
  renameTag: (old, newName) => call("rename_tag", { old, new: newName }),

  /** Переместить тег в списке (from → to). */
  moveTag: (from, to) => call("move_tag", { from, to }),

  /** Добавить клиента в справочник. */
  addClient: (name) => call("add_client", { name }),

  /** Удалить клиента по ID. */
  removeClient: (id) => call("remove_client", { id }),

  /** Удалить всех клиентов из справочника. */
  clearClients: () => call("clear_clients"),

  /** Переименовать клиента. */
  renameClient: (old, newName) => call("rename_client", { old, new: newName }),

  /** Переместить клиента в списке (from → to). */
  moveClient: (from, to) => call("move_client", { from, to }),

  /** Добавить статус в справочник (name + color). */
  addStatus: (name, color) => call("add_status", { name, color }),

  /** Удалить статус по ID (задачи получают пустой статус). */
  removeStatus: (id) => call("remove_status", { id }),

  /** Удалить все статусы из справочника (у задач статус сбрасывается). */
  clearStatuses: () => call("clear_statuses"),

  /** Переименовать статус (обновляется во всех задачах). */
  renameStatus: (old, newName) => call("rename_status", { old, new: newName }),

  /** Установить цвет статуса по ID. */
  setStatusColor: (id, color) => call("set_status_color", { id, color }),

  /** Переместить статус в списке (from → to). */
  moveStatus: (from, to) => call("move_status", { from, to }),
};

export function fmtTd(seconds) {
  const t = Math.max(0, Math.floor(seconds || 0));
  const h = Math.floor(t / 3600);
  const m = Math.floor((t % 3600) / 60);
  const s = t % 60;
  return [h, m, s].map((x) => String(x).padStart(2, "0")).join(":");
}