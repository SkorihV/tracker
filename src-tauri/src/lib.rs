//! Tauri-приложение: пути данных, бэкапы при запуске, регистрация команд.

mod commands;
mod tray;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, Manager};
use tracker_core::backups;
use tracker_core::store::{data_file, legacy_data_file, Store};

use commands::{base_dir, resolve_backups_dir, AppStore};

/// Найти прежний файл данных (папка data/ или файл рядом с exe), если нового ещё нет.
fn find_old_data(app: &AppHandle, new_data: &Path) -> Option<PathBuf> {
    if new_data.exists() {
        return None;
    }
    let base = base_dir(app);
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let mut candidates = vec![data_file(base.clone()), legacy_data_file(base)];
    if let Some(dir) = exe_dir {
        candidates.push(data_file(dir.clone()));
        candidates.push(legacy_data_file(dir));
    }
    candidates.into_iter().find(|p| p != new_data && p.exists())
}

/// Перенести файл данных на новое место. Если не вышло — вернуть путь, по
/// которому данные остались доступны, и текст предупреждения: данные не теряются.
fn migrate_old_data(app: &AppHandle, new_data: PathBuf) -> (PathBuf, Option<String>) {
    let Some(src) = find_old_data(app, &new_data) else {
        return (new_data, None);
    };
    if let Some(parent) = new_data.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return (src.clone(), Some(format!("Не удалось создать «{}»: {e}", parent.display())));
        }
    }
    // rename не работает между дисками — тогда копируем и удаляем исходный файл.
    let moved = std::fs::rename(&src, &new_data).or_else(|_| {
        std::fs::copy(&src, &new_data).and_then(|_| std::fs::remove_file(&src))
    });
    match moved {
        Ok(()) => (new_data, None),
        Err(e) => (
            src.clone(),
            Some(format!(
                "Не удалось перенести данные из «{}» в «{}»: {e}. Используется прежний файл.",
                src.display(),
                new_data.display()
            )),
        ),
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .on_window_event(tray::on_window_event)
        .setup(|app| {
            let base = base_dir(app.handle());
            let (data, warning) = migrate_old_data(app.handle(), data_file(base.clone()));
            if let Some(w) = warning {
                eprintln!("[tracker] {w}");
            }
            if let Some(data_dir) = data.parent() {
                backups::migrate_old_backups(data_dir);
            }

            let store = Store::open(data.clone());
            let backups_dir = resolve_backups_dir(&base, &store.settings.backup_dir);
            backups::create_backups(&data, &backups_dir);
            app.manage(AppStore(Mutex::new(store)));

            tray::setup(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::query,
            commands::report_preview,
            commands::create_report,
            commands::stats_preview,
            commands::create_stats,
            commands::export_json,
            commands::import_json,
            commands::create_task,
            commands::update_task,
            commands::set_task_status,
            commands::set_task_dates,
            commands::set_task_intervals,
            commands::task_action,
            commands::remove_tasks,
            commands::edit_dictionary,
            commands::set_settings,
            commands::set_our_car_color,
            commands::set_columns,
            commands::set_tray_prefs,
            commands::set_backups_dir,
            commands::default_backups_dir,
            commands::db_dir,
            commands::open_in_explorer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
