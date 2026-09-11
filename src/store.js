import { reactive } from "vue";
import { api } from "./api";

export const state = reactive({
  ready: false,
  loading: true,
  error: "",
  settings: { username: "", grouping: "none" },
  users: [],
  tags: [],
  clients: [],
  rows: [],
  totals: { count: 0, totalSeconds: 0, timeLabel: "00:00:00", byTag: [] },
  filter: {
    dateFrom: "",
    dateTo: "",
    tag: "",
    client: "",
    user: "",
    search: "",
  },
});

export async function init() {
  await refresh();
  state.loading = false;
}

export async function refresh() {
  try {
    const s = await api.getState();
    state.settings = s.settings;
    state.users = s.users || [];
    state.tags = s.tags || [];
    state.clients = s.clients || [];
    state.error = "";
  } catch (e) {
    state.error = String(e);
  }
}

export async function refreshQuery() {
  try {
    const q = await api.query(state.filter);
    state.rows = q.rows;
    state.totals = q.totals;
    state.error = "";
  } catch (e) {
    state.error = String(e);
  }
}

export async function applySettings(username, grouping) {
  await api.setSettings(username, grouping);
  state.settings.username = username;
  state.settings.grouping = grouping;
  await refresh();
  await refreshQuery();
}

export async function addTask(draft) {
  await api.createTask(draft);
  await refresh();
  await refreshQuery();
}

export async function updateTask(taskId, draft) {
  await api.updateTask({ taskId, ...draft });
  await refresh();
  await refreshQuery();
}

export async function startTask(id) {
  await api.startTask(id);
  await refreshQuery();
}

export async function pauseTask(id) {
  await api.pauseTask(id);
  await refreshQuery();
}

export async function resumeTask(id) {
  await api.resumeTask(id);
  await refreshQuery();
}

export async function completeTask(id) {
  await api.completeTask(id);
  await refreshQuery();
}

export async function removeTasks(ids) {
  await api.removeTasks(ids);
  await refresh();
  await refreshQuery();
}

export async function saveEntity(kind, payload) {
  // kind: user | tag | client ; payload: {action, ...}
  if (kind === "user") {
    if (payload.action === "add") await api.addUser(payload.name);
    else if (payload.action === "rename") await api.renameUser(payload.old, payload.name);
    else if (payload.action === "remove") await api.removeUser(payload.name);
  } else if (kind === "tag") {
    if (payload.action === "add") await api.addTag(payload.name);
    else if (payload.action === "rename") await api.renameTag(payload.old, payload.name);
    else if (payload.action === "remove") await api.removeTag(payload.id);
  } else if (kind === "client") {
    if (payload.action === "add") await api.addClient(payload.name);
    else if (payload.action === "rename") await api.renameClient(payload.old, payload.name);
    else if (payload.action === "remove") await api.removeClient(payload.id);
  }
  await refresh();
  await refreshQuery();
}