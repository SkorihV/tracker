//! Проверка совместимости с реальным data/time_tracker_v2_data.json:
//! загрузка, round-trip, фильтры, группировка, бэкапы, генерация task_id.
//! Запуск:  TT_TEST_DATA=<путь к json> cargo test -p tracker_core

use std::env;
use std::fs;
use std::path::Path;

use tracker_core::backups::{create_backups, migrate_old_backups};
use tracker_core::logic::{apply_filter, build_rows, build_totals, TaskFilter, TaskIdGen};
use tracker_core::models::{TaskDraft, TaskStatus};
use tracker_core::store::Store;

fn data_path() -> Option<std::path::PathBuf> {
    env::var("TT_TEST_DATA").ok().map(std::path::PathBuf::from)
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("tt_test_{tag}_{}", std::process::id()));
    if p.exists() {
        let _ = fs::remove_dir_all(&p);
    }
    let _ = fs::create_dir_all(&p);
    p
}

#[test]
fn loads_real_data() {
    let Some(src) = data_path() else {
        eprintln!("SKIP: TT_TEST_DATA not set");
        return;
    };
    let dir = temp_dir("load");
    let db = dir.join("data").join("time_tracker_v2_data.json");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    fs::copy(&src, &db).unwrap();

    let st = Store::open(db);
    println!("users={:?} tags={:?} clients={:?} tasks={}", st.users, st.tags, st.clients, st.tasks.len());
    println!("settings={:?}", st.settings);
    assert!(!st.tasks.is_empty(), "должны загрузиться задачи");
    assert!(st.users.iter().any(|u| u == &st.settings.username) || st.users.is_empty());
    for t in &st.tasks {
        let s = t.total_seconds(tracker_core::dt::now_naive());
        assert!(s >= 0.0);
        assert!(t.status == TaskStatus::Running || t.status == TaskStatus::Paused || t.status == TaskStatus::Completed);
        assert!(!t.task_id.is_empty());
    }
}

#[test]
fn round_trip_preserves_data() {
    let Some(src) = data_path() else {
        eprintln!("SKIP: TT_TEST_DATA not set");
        return;
    };
    let dir = temp_dir("roundtrip");
    let db = dir.join("data").join("time_tracker_v2_data.json");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    fs::copy(&src, &db).unwrap();

    let before: serde_json::Value = serde_json::from_str(&fs::read_to_string(&db).unwrap()).unwrap();
    {
        let st = Store::open(db.clone());
        st.save();
    }
    let after: serde_json::Value = serde_json::from_str(&fs::read_to_string(&db).unwrap()).unwrap();
    assert_eq!(
        before.pointer("/tasks").and_then(|x| x.as_array()).map(|a| a.len()),
        after.pointer("/tasks").and_then(|x| x.as_array()).map(|a| a.len()),
        "кол-во задач должно сохраниться"
    );
    assert_eq!(before["settings"], after["settings"], "settings должны совпасть");
    let users_before = before["users"].as_array().map(|a| a.len());
    let users_after = after["users"].as_array().map(|a| a.len());
    assert_eq!(users_before, users_after, "users должны совпасть");
}

#[test]
fn filters_group_and_totals() {
    let Some(src) = data_path() else {
        eprintln!("SKIP: TT_TEST_DATA not set");
        return;
    };
    let dir = temp_dir("query");
    let db = dir.join("data").join("time_tracker_v2_data.json");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    fs::copy(&src, &db).unwrap();

    let st = Store::open(db);
    let f = TaskFilter::default();
    let (rows, totals) = st.query(&f);
    assert_eq!(totals.count, st.tasks.len());
    assert!(rows.len() >= st.tasks.len());
    let _ = apply_filter(&st.tasks, &f, tracker_core::dt::now_naive());
    let _ = build_totals(&[]);

    // search по куску текста должен найти все задачи с этим словом
    let f2 = TaskFilter { search: "тест".into(), ..Default::default() };
    let hits = apply_filter(&st.tasks, &f2, tracker_core::dt::now_naive());
    println!("search 'тест' -> {} задач", hits.len());

    // группировка
    let grouped = build_rows(&hits, "day");
    assert!(grouped.iter().any(|r| matches!(r, tracker_core::logic::ViewRow::Group { .. })));
    let grp_client = build_rows(&hits, "client");
    assert!(grp_client.iter().any(|r| matches!(r, tracker_core::logic::ViewRow::Group { .. })));
}

#[test]
fn task_id_format() {
    let mut gen = TaskIdGen::new();
    let id = gen.next_id();
    let parts: Vec<&str> = id.split('-').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].len(), 8);
    assert_eq!(parts[1].len(), 3);
    assert_eq!(parts[2].len(), 4);
    let id2 = gen.next_id();
    assert_ne!(id, id2);
    eprintln!("sample task_id: {id}");
}

#[test]
fn backups_rotate() {
    let Some(src) = data_path() else {
        eprintln!("SKIP: TT_TEST_DATA not set");
        return;
    };
    let dir = temp_dir("backups");
    let data_dir = dir.join("data");
    fs::create_dir_all(&data_dir).unwrap();
    let db = data_dir.join("time_tracker_v2_data.json");
    fs::copy(&src, &db).unwrap();
    let bdir = dir.join("backups");

    // старый формат бэкапов — должен удалиться
    for i in 0..3 {
        fs::write(data_dir.join(format!("backup_2026090{i}_000000.json")), "x").unwrap();
    }
    migrate_old_backups(&data_dir);
    let left = fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("backup_"))
        .count();
    assert_eq!(left, 0, "старые backup_* должны удалиться");

    create_backups(&db, &bdir);
    let entries: Vec<String> = fs::read_dir(&bdir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    println!("создано бэкапов: {entries:?}");
    assert_eq!(entries.len(), 3, "должно быть daily+weekly+monthly");

    // повторный вызов в тот же день — новых файлов не добавляет
    create_backups(&db, &bdir);
    let n2 = fs::read_dir(&bdir).unwrap().count();
    assert_eq!(n2, 3, "повторный запуск не должен плодить бэкапы");
}

#[test]
fn store_mutations() {
    let Some(src) = data_path() else {
        eprintln!("SKIP: TT_TEST_DATA not set");
        return;
    };
    let dir = temp_dir("mut");
    let db = dir.join("data").join("time_tracker_v2_data.json");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    fs::copy(&src, &db).unwrap();

    let mut st = Store::open(db.clone());
    let before_n = st.tasks.len();

    let t = st.create_task(TaskDraft {
        user: "Тест 1".into(),
        order: "R-1".into(),
        tag: "".into(),
        client: "".into(),
        comment: "сквозной тест на Rust".into(),
    });
    assert_eq!(st.tasks.len(), before_n + 1);
    assert_eq!(t.status, "running");
    assert!(!t.task_id.is_empty());
    assert!(st.users.iter().any(|u| u == "Тест 1"));

    st.pause_task(&t.task_id);
    assert_eq!(st.tasks.last().unwrap().status, TaskStatus::Paused);
    st.resume_task(&t.task_id);
    assert_eq!(st.tasks.last().unwrap().status, TaskStatus::Running);
    st.complete_task(&t.task_id);
    assert_eq!(st.tasks.last().unwrap().status, TaskStatus::Completed);

    st.update_task(
        &t.task_id,
        TaskDraft { user: "Второй".into(), order: "R-2".into(), tag: "Тонкий".into(), client: "Клиент 1".into(), comment: "".into() },
    );
    let last = st.tasks.last().unwrap();
    assert_eq!(last.order, "R-2");
    assert_eq!(last.tag, "Тонкий");
    assert!(st.users.iter().any(|u| u == "Второй"), "пользователь добавляется автоматически");

    st.remove_task(&t.task_id);
    assert_eq!(st.tasks.len(), before_n);

    // повторное открытие из файла после мутаций
    let st2 = Store::open(db.clone());
    assert_eq!(st2.users.iter().filter(|u| **u == "Второй").count(), 1);
    assert!(Path::new(&db).exists());
}