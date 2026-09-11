//! Резервное копирование: папка backups/ рядом с data/.
//! daily — по одному на день (хранятся 7), weekly — один на неделю
//! (ключ = понедельник, хранятся 4), monthly — один на месяц (хранятся 3).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;

pub const CAP_DAILY: usize = 7;
pub const CAP_WEEKLY: usize = 4;
pub const CAP_MONTHLY: usize = 3;

/// Удалить старые бэкапы прежней версии логики (лежали в data/).
pub fn migrate_old_backups(data_dir: &Path) {
    if let Ok(entries) = fs::read_dir(data_dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("backup_") && name.ends_with(".json") {
                let _ = fs::remove_file(e.path());
            }
        }
    }
}

/// Создать бэкапы всех категорий (если за период ещё нет) + ротация.
pub fn create_backups(data_path: &Path, backup_dir: &Path) {
    if !data_path.exists() {
        return;
    }
    if fs::create_dir_all(backup_dir).is_err() {
        return;
    }
    let today = Local::now().date_naive();
    let monday = crate::dt::current_monday();
    backup_category(
        data_path,
        backup_dir,
        "daily",
        today.format("%Y%m%d").to_string(),
        CAP_DAILY,
    );
    backup_category(
        data_path,
        backup_dir,
        "weekly",
        monday.format("%Y%m%d").to_string(),
        CAP_WEEKLY,
    );
    backup_category(
        data_path,
        backup_dir,
        "monthly",
        today.format("%Y%m").to_string(),
        CAP_MONTHLY,
    );
}

fn list_category(backup_dir: &Path, cat: &str) -> Vec<PathBuf> {
    let prefix = format!("{cat}_");
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(backup_dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with(&prefix) && name.ends_with(".json") {
                out.push(e.path());
            }
        }
    }
    out
}

fn backup_category(data_path: &Path, backup_dir: &Path, cat: &str, key: String, cap: usize) {
    let target = backup_dir.join(format!("{cat}_{key}.json"));
    if !target.exists() {
        let _ = fs::copy(data_path, &target);
    }
    let mut files = list_category(backup_dir, cat);
    files.sort();
    let n = files.len();
    if n > cap {
        for f in &files[..n - cap] {
            let _ = fs::remove_file(f);
        }
    }
}