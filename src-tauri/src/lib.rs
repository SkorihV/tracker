//! Tauri-приложение: регистрация команд, пути данных, бэкапы при запуске.

mod commands;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};
use tracker_core::backups;
use tracker_core::logic::TaskFilter;
use tracker_core::models::{ColumnPref, TaskDraft};
use tracker_core::report;
use tracker_core::store::Store;

use commands::{AppStore, QueryView};

/// Базовый каталог данных: TT_BASE_DIR (dev) либо AppData приложения.
fn base_dir(app: &AppHandle) -> PathBuf {
    if let Ok(d) = std::env::var("TT_BASE_DIR") {
        return PathBuf::from(d);
    }
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Каталог бэкапов: пользовательский (из настроек) либо base/backups по умолчанию.
fn resolve_backups_dir(base: &Path, user_dir: &str) -> PathBuf {
    let d = user_dir.trim();
    if d.is_empty() {
        base.join("backups")
    } else {
        PathBuf::from(d)
    }
}

/// Перенос данных из прежней локации (папка data/ и файл рядом с exe) в новую.
fn migrate_old_data(app: &AppHandle, new_data: &Path) {
    if new_data.exists() {
        return;
    }
    let bases = [
        tracker_core::store::data_file(base_dir(app).clone()),
        tracker_core::store::legacy_data_file(base_dir(app)),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|dir| tracker_core::store::data_file(dir))
            .unwrap_or_default(),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .map(|dir| tracker_core::store::legacy_data_file(dir))
            .unwrap_or_default(),
    ];
    for src in bases {
        if src.exists() {
            if let Some(parent) = new_data.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::rename(&src, new_data);
            return;
        }
    }
}

#[tauri::command]
fn get_state(s: State<'_, AppStore>) -> Result<tracker_core::models::AppState, String> {
    s.0.lock().map_err(|e| e.to_string()).map(|st| st.state())
}

#[tauri::command]
fn query(s: State<'_, AppStore>, filter: TaskFilter) -> Result<QueryView, String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    let (rows, totals) = st.query(&filter);
    Ok(QueryView { rows, totals })
}

#[tauri::command]
fn report_preview(s: State<'_, AppStore>, filter: TaskFilter) -> Result<report::ReportPreview, String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    Ok(report::report_preview(&st.tasks, &filter))
}

#[tauri::command]
fn create_report(
    app: AppHandle,
    s: State<'_, AppStore>,
    format: String,
    filter: TaskFilter,
    date_from: String,
    date_to: String,
    path: Option<String>,
) -> Result<String, String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    let accent = st.settings.accent_color.clone();
    let saved = match path {
        Some(p) if !p.is_empty() => {
            report::write_report_to(std::path::Path::new(&p), &format, &st.tasks, &filter, &date_from, &date_to, &accent, &st.settings.columns)?
        }
        _ => {
            let base = base_dir(&app);
            report::write_report(&base, &format, &st.tasks, &filter, &date_from, &date_to, &accent, &st.settings.columns)?
        }
    };
    Ok(saved.to_string_lossy().to_string())
}

#[tauri::command]
fn stats_preview(s: State<'_, AppStore>, filter: TaskFilter) -> Result<report::Stats, String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    Ok(report::statistics(&st.tasks, &filter))
}

#[tauri::command]
fn create_stats(
    app: AppHandle,
    s: State<'_, AppStore>,
    format: String,
    filter: TaskFilter,
    date_from: String,
    date_to: String,
    path: Option<String>,
) -> Result<String, String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    let accent = st.settings.accent_color.clone();
    let saved = match path {
        Some(p) if !p.is_empty() => {
            report::write_stats_to(std::path::Path::new(&p), &format, &st.tasks, &filter, &date_from, &date_to, &accent)?
        }
        _ => {
            let base = base_dir(&app);
            report::write_stats(&base, &format, &st.tasks, &filter, &date_from, &date_to, &accent)?
        }
    };
    Ok(saved.to_string_lossy().to_string())
}

#[tauri::command]
fn export_json(
    s: State<'_, AppStore>,
    path: String,
    export_tasks: bool,
    export_tags: bool,
    export_clients: bool,
    export_users: bool,
    export_statuses: bool,
) -> Result<(), String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    tracker_core::Store::export_json_file(&st, std::path::Path::new(&path), export_tasks, export_tags, export_clients, export_users, export_statuses)
}

#[tauri::command]
fn import_json(
    s: State<'_, AppStore>,
    path: String,
    import_tasks: bool,
    import_tags: bool,
    import_clients: bool,
    import_users: bool,
    import_statuses: bool,
) -> Result<(), String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    tracker_core::Store::import_json_file(&mut st, std::path::Path::new(&path), import_tasks, import_tags, import_clients, import_users, import_statuses)
}

#[tauri::command]
fn create_task(s: State<'_, AppStore>, user: String, orders: Vec<String>, tags: Vec<String>, client: String, comment: String, custom_status: String, our_car: bool) -> Result<tracker_core::models::AppTask, String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    Ok(st.create_task(TaskDraft { user, orders, tags, client, comment, custom_status, our_car }))
}

#[tauri::command]
fn update_task(s: State<'_, AppStore>, task_id: String, user: String, orders: Vec<String>, tags: Vec<String>, client: String, comment: String, custom_status: String, our_car: bool) -> Result<(), String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    st.update_task(&task_id, TaskDraft { user, orders, tags, client, comment, custom_status, our_car });
    Ok(())
}

#[tauri::command]
fn set_task_status(s: State<'_, AppStore>, task_id: String, status: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.set_task_status(&task_id, status);
    Ok(())
}

#[tauri::command]
fn set_task_dates(s: State<'_, AppStore>, task_id: String, start: String, end: String) -> Result<(), String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    st.set_task_dates(&task_id, start, end)
}

#[tauri::command]
fn set_task_intervals(s: State<'_, AppStore>, task_id: String, lines: Vec<String>) -> Result<(), String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    st.set_task_intervals(&task_id, lines)
}

#[tauri::command]
fn start_task(s: State<'_, AppStore>, task_id: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.start_task(&task_id);
    Ok(())
}

#[tauri::command]
fn pause_task(s: State<'_, AppStore>, task_id: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.pause_task(&task_id);
    Ok(())
}

#[tauri::command]
fn resume_task(s: State<'_, AppStore>, task_id: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.resume_task(&task_id);
    Ok(())
}

#[tauri::command]
fn complete_task(s: State<'_, AppStore>, task_id: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.complete_task(&task_id);
    Ok(())
}

#[tauri::command]
fn remove_tasks(s: State<'_, AppStore>, ids: Vec<String>) -> Result<(), String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    for id in ids {
        st.remove_task(&id);
    }
    Ok(())
}

#[tauri::command]
fn set_settings(s: State<'_, AppStore>, username: String, grouping: String, accent_color: String) -> Result<(), String> {
    s.0.lock()
        .map_err(|e| e.to_string())?
        .set_settings(username, grouping, accent_color);
    Ok(())
}

#[tauri::command]
fn set_our_car_color(s: State<'_, AppStore>, color: String) -> Result<(), String> {
    s.0.lock()
        .map_err(|e| e.to_string())?
        .set_our_car_color(color);
    Ok(())
}

#[tauri::command]
fn set_columns(s: State<'_, AppStore>, columns: Vec<ColumnPref>) -> Result<(), String> {
    s.0.lock()
        .map_err(|e| e.to_string())?
        .set_columns(columns);
    Ok(())
}

#[tauri::command]
fn set_backups_dir(s: State<'_, AppStore>, dir: String) -> Result<(), String> {
    s.0.lock()
        .map_err(|e| e.to_string())?
        .set_backups_dir(dir);
    Ok(())
}

#[tauri::command]
fn default_backups_dir(app: AppHandle) -> Result<String, String> {
    Ok(resolve_backups_dir(&base_dir(&app), "")
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn add_user(s: State<'_, AppStore>, name: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.add_user(&name);
    Ok(())
}

#[tauri::command]
fn remove_user(s: State<'_, AppStore>, name: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.remove_user(&name);
    Ok(())
}

#[tauri::command]
fn clear_users(s: State<'_, AppStore>) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.clear_users();
    Ok(())
}

#[tauri::command]
fn rename_user(s: State<'_, AppStore>, old: String, new: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.rename_user(&old, &new);
    Ok(())
}

#[tauri::command]
fn move_user(s: State<'_, AppStore>, from: usize, to: usize) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.move_user(from, to);
    Ok(())
}

#[tauri::command]
fn add_tag(s: State<'_, AppStore>, name: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.add_tag(&name);
    Ok(())
}

#[tauri::command]
fn remove_tag(s: State<'_, AppStore>, id: u64) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.remove_tag(id);
    Ok(())
}

#[tauri::command]
fn clear_tags(s: State<'_, AppStore>) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.clear_tags();
    Ok(())
}

#[tauri::command]
fn rename_tag(s: State<'_, AppStore>, old: String, new: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.rename_tag(&old, &new);
    Ok(())
}

#[tauri::command]
fn move_tag(s: State<'_, AppStore>, from: usize, to: usize) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.move_tag(from, to);
    Ok(())
}

#[tauri::command]
fn add_client(s: State<'_, AppStore>, name: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.add_client(&name);
    Ok(())
}

#[tauri::command]
fn remove_client(s: State<'_, AppStore>, id: u64) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.remove_client(id);
    Ok(())
}

#[tauri::command]
fn clear_clients(s: State<'_, AppStore>) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.clear_clients();
    Ok(())
}

#[tauri::command]
fn rename_client(s: State<'_, AppStore>, old: String, new: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.rename_client(&old, &new);
    Ok(())
}

#[tauri::command]
fn move_client(s: State<'_, AppStore>, from: usize, to: usize) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.move_client(from, to);
    Ok(())
}

// ---------- Статусы задачи (справочник) ----------

#[tauri::command]
fn add_status(s: State<'_, AppStore>, name: String, color: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.add_status(&name, &color);
    Ok(())
}

#[tauri::command]
fn remove_status(s: State<'_, AppStore>, id: u64) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.remove_status(id);
    Ok(())
}

#[tauri::command]
fn clear_statuses(s: State<'_, AppStore>) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.clear_statuses();
    Ok(())
}

#[tauri::command]
fn rename_status(s: State<'_, AppStore>, old: String, new: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.rename_status(&old, &new);
    Ok(())
}

#[tauri::command]
fn set_status_color(s: State<'_, AppStore>, id: u64, color: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.set_status_color(id, color);
    Ok(())
}

#[tauri::command]
fn move_status(s: State<'_, AppStore>, from: usize, to: usize) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.move_status(from, to);
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let base = base_dir(app.handle());
            let data = tracker_core::store::data_file(base.clone());
            migrate_old_data(app.handle(), &data);
            let data_dir = data.parent().map(|p| p.to_path_buf()).unwrap_or_default();
            backups::migrate_old_backups(&data_dir);

            let store = AppStore(Mutex::new(Store::open(data.clone())));
            {
                let st = store.0.lock().map_err(|e| e.to_string())?;
                let backups_dir = resolve_backups_dir(&base, &st.settings.backup_dir);
                backups::create_backups(&data, &backups_dir);
            }
            app.manage(store);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_state,
            query,
            report_preview,
            create_report,
            stats_preview,
            create_stats,
            export_json,
            import_json,
            create_task,
            update_task,
            set_task_status,
            set_task_dates,
            set_task_intervals,
            start_task,
            pause_task,
            resume_task,
            complete_task,
            remove_tasks,
            set_settings,
            set_our_car_color,
            set_columns,
            set_backups_dir,
            default_backups_dir,
            add_user,
            remove_user,
            clear_users,
            rename_user,
            move_user,
            add_tag,
            remove_tag,
            clear_tags,
            rename_tag,
            move_tag,
            add_client,
            remove_client,
            clear_clients,
            rename_client,
            move_client,
            add_status,
            remove_status,
            clear_statuses,
            rename_status,
            set_status_color,
            move_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}