//! Поведение хранилища без внешних данных: ошибки записи, справочники,
//! массовое удаление, напоминания. Каждый тест работает во временном каталоге.

use std::fs;
use std::path::PathBuf;

use tracker_core::dt::now_naive;
use tracker_core::models::{TaskDraft, TaskStatus};
use tracker_core::{DictAction, DictKind, Store, TaskAction};

fn temp_db(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tt_behavior_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir.join("data").join("db.json")
}

fn draft(user: &str, tag: &str, client: &str) -> TaskDraft {
    TaskDraft {
        user: user.into(),
        orders: vec!["R-1".into()],
        tags: if tag.is_empty() { vec![] } else { vec![tag.into()] },
        client: client.into(),
        comment: String::new(),
        custom_status: String::new(),
        our_car: false,
    }
}

fn add(name: &str) -> DictAction {
    DictAction::Add { name: name.into(), color: None }
}

#[test]
fn save_reports_write_errors() {
    let db = temp_db("save_err");
    let mut st = Store::open(db.clone());
    // На месте каталога данных лежит обычный файл — запись невозможна.
    fs::create_dir_all(db.parent().unwrap().parent().unwrap()).unwrap();
    fs::write(db.parent().unwrap(), b"not a dir").unwrap();
    assert!(st.save().is_err());
    assert!(st.create_task(draft("A", "", "")).is_err());
}

#[test]
fn save_leaves_no_temp_file() {
    let db = temp_db("tmp");
    let st = Store::open(db.clone());
    st.save().unwrap();
    assert!(db.exists());
    assert!(!db.with_extension("json.tmp").exists());
}

#[test]
fn dictionary_add_ignores_duplicates_and_blanks() {
    let mut st = Store::open(temp_db("dict_add"));
    st.edit_dictionary(DictKind::Tag, add("  Срочно ")).unwrap();
    st.edit_dictionary(DictKind::Tag, add("Срочно")).unwrap();
    st.edit_dictionary(DictKind::Tag, add("   ")).unwrap();
    assert_eq!(st.tags.len(), 1);
    assert_eq!(st.tags[0].name, "Срочно");
    assert_eq!(st.next_tag_id, 2);
}

#[test]
fn dictionary_rename_cascades_to_tasks() {
    let mut st = Store::open(temp_db("dict_rename"));
    let t = st.create_task(draft("Иван", "Альфа", "Клиент")).unwrap();
    let rename = |old: &str, new: &str| DictAction::Rename { old: old.into(), name: new.into() };
    st.edit_dictionary(DictKind::User, rename("Иван", "Пётр")).unwrap();
    st.edit_dictionary(DictKind::Client, rename("Клиент", "Заказчик")).unwrap();
    st.edit_dictionary(DictKind::Tag, rename("Альфа", "Бета")).unwrap();
    let task = st.tasks.iter().find(|x| x.task_id == t.task_id).unwrap();
    assert_eq!(task.user, "Пётр");
    assert_eq!(task.client, "Заказчик");
    assert_eq!(task.tags, vec!["Бета"]);
    assert!(st.users.contains(&"Пётр".to_string()));
}

#[test]
fn dictionary_remove_move_clear() {
    let mut st = Store::open(temp_db("dict_misc"));
    for n in ["a", "b", "c"] {
        st.edit_dictionary(DictKind::Client, add(n)).unwrap();
    }
    st.edit_dictionary(DictKind::Client, DictAction::Move { from: 0, to: 2 }).unwrap();
    let names: Vec<_> = st.clients.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["b", "c", "a"]);

    let id = st.clients[0].id;
    st.edit_dictionary(DictKind::Client, DictAction::Remove { id: Some(id), name: None }).unwrap();
    assert_eq!(st.clients.len(), 2);

    st.edit_dictionary(DictKind::Client, DictAction::Clear).unwrap();
    assert!(st.clients.is_empty());

    st.edit_dictionary(DictKind::User, add("Аня")).unwrap();
    st.edit_dictionary(DictKind::User, DictAction::Remove { id: None, name: Some("Аня".into()) }).unwrap();
    assert!(st.users.is_empty());
}

#[test]
fn status_color_and_removal_clear_task_status() {
    let mut st = Store::open(temp_db("status"));
    st.add_status("Новая", "blue").unwrap();
    let id = st.statuses[0].id;
    st.edit_dictionary(DictKind::Status, DictAction::Color { id, color: "red".into() }).unwrap();
    assert_eq!(st.statuses[0].color, "red");
    let t = st.create_task(draft("A", "", "")).unwrap();
    st.set_task_status(&t.task_id, "Новая".into()).unwrap();
    st.remove_status(id).unwrap();
    assert_eq!(st.tasks[0].custom_status, "");
}

#[test]
fn remove_tasks_removes_many_at_once() {
    let mut st = Store::open(temp_db("remove_many"));
    let ids: Vec<String> = (0..3)
        .map(|_| st.create_task(draft("A", "", "")).unwrap().task_id)
        .collect();
    st.remove_tasks(&ids[..2]).unwrap();
    assert_eq!(st.tasks.len(), 1);
    assert_eq!(st.tasks[0].task_id, ids[2]);
}

#[test]
fn task_actions_follow_timer_rules() {
    let mut st = Store::open(temp_db("actions"));
    let t = st.create_task(draft("A", "", "")).unwrap();
    st.task_action(&t.task_id, TaskAction::Pause).unwrap();
    assert_eq!(st.tasks[0].status, TaskStatus::Paused);
    st.task_action(&t.task_id, TaskAction::Resume).unwrap();
    assert_eq!(st.tasks[0].status, TaskStatus::Running);
    // Повторный resume не открывает второй диапазон.
    st.task_action(&t.task_id, TaskAction::Resume).unwrap();
    assert_eq!(st.tasks[0].intervals.iter().filter(|i| i.stop.is_none()).count(), 1);
    st.task_action(&t.task_id, TaskAction::Complete).unwrap();
    assert_eq!(st.tasks[0].status, TaskStatus::Completed);
}

#[test]
fn running_elapsed_lists_only_running_tasks() {
    let mut st = Store::open(temp_db("elapsed"));
    let a = st.create_task(draft("A", "", "")).unwrap();
    let b = st.create_task(draft("B", "", "")).unwrap();
    st.pause_task(&b.task_id).unwrap();
    let later = now_naive() + chrono::Duration::hours(5);
    let running = st.running_elapsed(later);
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].0.task_id, a.task_id);
    assert!(running[0].1 >= 5 * 3600);
}

#[test]
fn tray_prefs_are_clamped_and_persisted() {
    let db = temp_db("tray");
    let mut st = Store::open(db.clone());
    st.set_tray_prefs(true, 10_000).unwrap();
    let st2 = Store::open(db);
    assert!(st2.settings.minimize_to_tray);
    assert_eq!(st2.settings.reminder_hours, tracker_core::models::MAX_REMINDER_HOURS);
}

#[test]
fn backups_dir_length_is_validated() {
    let mut st = Store::open(temp_db("backup_dir"));
    assert!(st.set_backups_dir("x".repeat(600)).is_err());
    assert!(st.set_backups_dir("  C:/backups ".into()).is_ok());
    assert_eq!(st.settings.backup_dir, "C:/backups");
}
