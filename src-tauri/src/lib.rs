//! Tauri-приложение: регистрация команд, пути данных, бэкапы при запуске.

mod commands;

use std::sync::Mutex;

use tauri::State;
use tracker_core::backups;
use tracker_core::logic::TaskFilter;
use tracker_core::models::TaskDraft;
use tracker_core::report;
use tracker_core::store::Store;

use commands::{AppStore, QueryView};

fn base_dir() -> std::path::PathBuf {
    if let Ok(d) = std::env::var("TT_BASE_DIR") {
        return std::path::PathBuf::from(d);
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
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
    s: State<'_, AppStore>,
    format: String,
    filter: TaskFilter,
    date_from: String,
    date_to: String,
) -> Result<String, String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    let base = base_dir();
    let path = report::write_report(&base, &format, &st.tasks, &filter, &date_from, &date_to)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn stats_preview(s: State<'_, AppStore>, filter: TaskFilter) -> Result<report::Stats, String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    Ok(report::statistics(&st.tasks, &filter))
}

#[tauri::command]
fn create_stats(
    s: State<'_, AppStore>,
    format: String,
    filter: TaskFilter,
    date_from: String,
    date_to: String,
) -> Result<String, String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    let base = base_dir();
    let path = report::write_stats(&base, &format, &st.tasks, &filter, &date_from, &date_to)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn export_json(
    s: State<'_, AppStore>,
    path: String,
    export_tasks: bool,
    export_tags: bool,
    export_clients: bool,
    export_users: bool,
) -> Result<(), String> {
    let st = s.0.lock().map_err(|e| e.to_string())?;
    tracker_core::Store::export_json_file(&st, std::path::Path::new(&path), export_tasks, export_tags, export_clients, export_users)
}

#[tauri::command]
fn import_json(
    s: State<'_, AppStore>,
    path: String,
    import_tasks: bool,
    import_tags: bool,
    import_clients: bool,
    import_users: bool,
) -> Result<(), String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    tracker_core::Store::import_json_file(&mut st, std::path::Path::new(&path), import_tasks, import_tags, import_clients, import_users)
}

#[tauri::command]
fn create_task(s: State<'_, AppStore>, user: String, order: String, tags: Vec<String>, client: String, comment: String, custom_status: String) -> Result<tracker_core::models::AppTask, String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    Ok(st.create_task(TaskDraft { user, order, tags, client, comment, custom_status }))
}

#[tauri::command]
fn update_task(s: State<'_, AppStore>, task_id: String, user: String, order: String, tags: Vec<String>, client: String, comment: String, custom_status: String) -> Result<(), String> {
    let mut st = s.0.lock().map_err(|e| e.to_string())?;
    st.update_task(&task_id, TaskDraft { user, order, tags, client, comment, custom_status });
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
fn set_settings(s: State<'_, AppStore>, username: String, grouping: String) -> Result<(), String> {
    s.0.lock().map_err(|e| e.to_string())?.set_settings(username, grouping);
    Ok(())
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
    let base = base_dir();
    let data = tracker_core::store::data_file(base.clone());
    let legacy = tracker_core::store::legacy_data_file(base.clone());

    // миграция легаси-файла: base/time_tracker_v2_data.json -> base/data/…
    if !data.exists() {
        if let Some(parent) = data.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if legacy.exists() {
            let _ = std::fs::rename(&legacy, &data);
        }
        let _ = legacy; // (необязательно для загрузки ниже)
    }

    let store = AppStore(Mutex::new(Store::open(data.clone())));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(store)
        .setup(move |app| {
            let _ = app;
            let base2 = base_dir();
            let data2 = tracker_core::store::data_file(base2.clone());
            let data_dir = data2.parent().map(|p| p.to_path_buf()).unwrap_or_default();
            let backups_dir = base2.join("backups");
            backups::migrate_old_backups(&data_dir);
            backups::create_backups(&data2, &backups_dir);
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