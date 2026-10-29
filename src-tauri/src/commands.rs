//! Команды Tauri. Каждая берёт блокировку хранилища через `with_store`
//! и возвращает ошибку (в том числе ошибку записи на диск) интерфейсу.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tracker_core::logic::{TaskFilter, Totals, ViewRow};
use tracker_core::models::{AppState, AppTask, ColumnPref, TaskDraft};
use tracker_core::report;
use tracker_core::store::{self, Store};
use tracker_core::{DictAction, DictKind, TaskAction};

pub struct AppStore(pub Mutex<Store>);

impl AppStore {
    pub fn lock(&self) -> Result<MutexGuard<'_, Store>, String> {
        self.0.lock().map_err(|e| e.to_string())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryView {
    pub rows: Vec<ViewRow>,
    pub totals: Totals,
}

/// Базовый каталог данных: TT_BASE_DIR (dev) либо AppData приложения.
pub fn base_dir(app: &AppHandle) -> PathBuf {
    if let Ok(d) = std::env::var("TT_BASE_DIR") {
        return PathBuf::from(d);
    }
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Каталог бэкапов: пользовательский (из настроек) либо base/backups по умолчанию.
pub fn resolve_backups_dir(base: &Path, user_dir: &str) -> PathBuf {
    let d = user_dir.trim();
    if d.is_empty() {
        base.join("backups")
    } else {
        PathBuf::from(d)
    }
}

// ---------------------------------------------------------------------
// Чтение
// ---------------------------------------------------------------------

#[tauri::command]
pub fn get_state(s: State<'_, AppStore>) -> Result<AppState, String> {
    Ok(s.lock()?.state())
}

#[tauri::command]
pub fn query(s: State<'_, AppStore>, filter: TaskFilter) -> Result<QueryView, String> {
    let (rows, totals) = s.lock()?.query(&filter);
    Ok(QueryView { rows, totals })
}

// ---------------------------------------------------------------------
// Отчёты и статистика
// ---------------------------------------------------------------------

#[tauri::command]
pub fn report_preview(s: State<'_, AppStore>, filter: TaskFilter) -> Result<report::ReportPreview, String> {
    Ok(report::report_preview(&s.lock()?.tasks, &filter))
}

#[tauri::command]
pub fn stats_preview(s: State<'_, AppStore>, filter: TaskFilter) -> Result<report::Stats, String> {
    Ok(report::statistics(&s.lock()?.tasks, &filter))
}

#[tauri::command]
pub fn create_report(
    app: AppHandle,
    s: State<'_, AppStore>,
    format: String,
    filter: TaskFilter,
    date_from: String,
    date_to: String,
    path: Option<String>,
) -> Result<String, String> {
    let st = s.lock()?;
    let accent = &st.settings.accent_color;
    let saved = match path.filter(|p| !p.is_empty()) {
        Some(p) => report::write_report_to(Path::new(&p), &format, &st.tasks, &filter, &date_from, &date_to, accent, &st.settings.columns)?,
        None => report::write_report(&base_dir(&app), &format, &st.tasks, &filter, &date_from, &date_to, accent, &st.settings.columns)?,
    };
    Ok(saved.to_string_lossy().to_string())
}

#[tauri::command]
pub fn create_stats(
    app: AppHandle,
    s: State<'_, AppStore>,
    format: String,
    filter: TaskFilter,
    date_from: String,
    date_to: String,
    path: Option<String>,
) -> Result<String, String> {
    let st = s.lock()?;
    let accent = &st.settings.accent_color;
    let saved = match path.filter(|p| !p.is_empty()) {
        Some(p) => report::write_stats_to(Path::new(&p), &format, &st.tasks, &filter, &date_from, &date_to, accent)?,
        None => report::write_stats(&base_dir(&app), &format, &st.tasks, &filter, &date_from, &date_to, accent)?,
    };
    Ok(saved.to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_json(
    s: State<'_, AppStore>,
    path: String,
    export_tasks: bool,
    export_tags: bool,
    export_clients: bool,
    export_users: bool,
    export_statuses: bool,
) -> Result<(), String> {
    let st = s.lock()?;
    Store::export_json_file(&st, Path::new(&path), export_tasks, export_tags, export_clients, export_users, export_statuses)
}

#[tauri::command]
pub fn import_json(
    s: State<'_, AppStore>,
    path: String,
    import_tasks: bool,
    import_tags: bool,
    import_clients: bool,
    import_users: bool,
    import_statuses: bool,
) -> Result<(), String> {
    let mut st = s.lock()?;
    Store::import_json_file(&mut st, Path::new(&path), import_tasks, import_tags, import_clients, import_users, import_statuses)
}

// ---------------------------------------------------------------------
// Задачи
// ---------------------------------------------------------------------

#[tauri::command]
pub fn create_task(s: State<'_, AppStore>, draft: TaskDraft) -> Result<AppTask, String> {
    s.lock()?.create_task(draft)
}

#[tauri::command]
pub fn update_task(s: State<'_, AppStore>, task_id: String, draft: TaskDraft) -> Result<(), String> {
    s.lock()?.update_task(&task_id, draft)
}

#[tauri::command]
pub fn set_task_status(s: State<'_, AppStore>, task_id: String, status: String) -> Result<(), String> {
    s.lock()?.set_task_status(&task_id, status)
}

#[tauri::command]
pub fn set_task_dates(s: State<'_, AppStore>, task_id: String, start: String, end: String) -> Result<(), String> {
    s.lock()?.set_task_dates(&task_id, start, end)
}

#[tauri::command]
pub fn set_task_intervals(s: State<'_, AppStore>, task_id: String, lines: Vec<String>) -> Result<(), String> {
    s.lock()?.set_task_intervals(&task_id, lines)
}

/// Таймер задачи: start | pause | resume | complete.
#[tauri::command]
pub fn task_action(s: State<'_, AppStore>, task_id: String, action: TaskAction) -> Result<(), String> {
    s.lock()?.task_action(&task_id, action)
}

#[tauri::command]
pub fn remove_tasks(s: State<'_, AppStore>, ids: Vec<String>) -> Result<(), String> {
    s.lock()?.remove_tasks(&ids)
}

// ---------------------------------------------------------------------
// Справочники и настройки
// ---------------------------------------------------------------------

/// Любая правка справочника: kind = user | tag | client | status.
#[tauri::command]
pub fn edit_dictionary(s: State<'_, AppStore>, kind: DictKind, action: DictAction) -> Result<(), String> {
    s.lock()?.edit_dictionary(kind, action)
}

#[tauri::command]
pub fn set_settings(s: State<'_, AppStore>, username: String, grouping: String, accent_color: String) -> Result<(), String> {
    s.lock()?.set_settings(username, grouping, accent_color)
}

#[tauri::command]
pub fn set_our_car_color(s: State<'_, AppStore>, color: String) -> Result<(), String> {
    s.lock()?.set_our_car_color(color)
}

#[tauri::command]
pub fn set_columns(s: State<'_, AppStore>, columns: Vec<ColumnPref>) -> Result<(), String> {
    s.lock()?.set_columns(columns)
}

#[tauri::command]
pub fn set_tray_prefs(s: State<'_, AppStore>, minimize_to_tray: bool, reminder_hours: u32) -> Result<(), String> {
    s.lock()?.set_tray_prefs(minimize_to_tray, reminder_hours)
}

#[tauri::command]
pub fn set_backups_dir(s: State<'_, AppStore>, dir: String) -> Result<(), String> {
    s.lock()?.set_backups_dir(dir)
}

// ---------------------------------------------------------------------
// Папки
// ---------------------------------------------------------------------

#[tauri::command]
pub fn default_backups_dir(app: AppHandle) -> String {
    resolve_backups_dir(&base_dir(&app), "").to_string_lossy().to_string()
}

/// Папка, в которой хранится файл БД (base/data).
#[tauri::command]
pub fn db_dir(app: AppHandle) -> String {
    let file = store::data_file(base_dir(&app));
    file.parent().unwrap_or(&file).to_string_lossy().to_string()
}

/// Открыть папку в проводнике (создаёт её, если она ещё не существует).
#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    let dir = PathBuf::from(path);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Не удалось создать папку: {e}"))?;
    let opener = if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(opener)
        .arg(&dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Не удалось открыть папку: {e}"))
}
