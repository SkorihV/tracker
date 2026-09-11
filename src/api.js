import { invoke } from "@tauri-apps/api/core";

async function call(cmd, args) {
  try {
    return await invoke(cmd, args);
  } catch (e) {
    console.error(`[${cmd}]`, e);
    throw e;
  }
}

export const api = {
  getState: () => call("get_state"),
  query: (filter) => call("query", { filter }),
  reportPreview: (filter) => call("report_preview", { filter }),
  createReport: (format, filter, dateFrom, dateTo) =>
    call("create_report", { format, filter, dateFrom, dateTo }),
  statsPreview: (filter) => call("stats_preview", { filter }),
  createStats: (format, filter, dateFrom, dateTo) =>
    call("create_stats", { format, filter, dateFrom, dateTo }),
  exportJson: (path, opts) =>
    call("export_json", {
      path,
      exportTasks: opts.tasks,
      exportTags: opts.tags,
      exportClients: opts.clients,
      exportUsers: opts.users,
    }),
  importJson: (path, opts) =>
    call("import_json", {
      path,
      importTasks: opts.tasks,
      importTags: opts.tags,
      importClients: opts.clients,
      importUsers: opts.users,
    }),
  createTask: (d) => call("create_task", d),
  updateTask: (d) => call("update_task", d),
  startTask: (taskId) => call("start_task", { taskId }),
  pauseTask: (taskId) => call("pause_task", { taskId }),
  resumeTask: (taskId) => call("resume_task", { taskId }),
  completeTask: (taskId) => call("complete_task", { taskId }),
  removeTasks: (ids) => call("remove_tasks", { ids }),
  setSettings: (username, grouping) => call("set_settings", { username, grouping }),

  addUser: (name) => call("add_user", { name }),
  removeUser: (name) => call("remove_user", { name }),
  renameUser: (old, newName) => call("rename_user", { old, new: newName }),

  addTag: (name) => call("add_tag", { name }),
  removeTag: (id) => call("remove_tag", { id }),
  renameTag: (old, newName) => call("rename_tag", { old, new: newName }),

  addClient: (name) => call("add_client", { name }),
  removeClient: (id) => call("remove_client", { id }),
  renameClient: (old, newName) => call("rename_client", { old, new: newName }),
};

export function fmtTd(seconds) {
  const t = Math.max(0, Math.floor(seconds || 0));
  const h = Math.floor(t / 3600);
  const m = Math.floor((t % 3600) / 60);
  const s = t % 60;
  return [h, m, s].map((x) => String(x).padStart(2, "0")).join(":");
}