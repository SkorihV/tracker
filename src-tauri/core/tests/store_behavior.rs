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

fn timed_task(st: &mut Store, tag: &str, lines: &[&str]) -> String {
    let id = st.create_task(draft("A", tag, "")).unwrap().task_id;
    st.set_task_intervals(&id, lines.iter().map(|l| l.to_string()).collect()).unwrap();
    id
}

fn day_filter() -> tracker_core::TaskFilter {
    tracker_core::TaskFilter {
        date_from: "01.01.2026".into(),
        date_to: "01.01.2026".into(),
        ..Default::default()
    }
}

#[test]
fn actual_time_counts_overlaps_once() {
    let mut st = Store::open(temp_db("overlap"));
    // Две полностью параллельные задачи по часу и одна, пересекающаяся на полчаса.
    timed_task(&mut st, "", &["01.01.2026 10:00 — 01.01.2026 11:00"]);
    timed_task(&mut st, "", &["01.01.2026 10:00 — 01.01.2026 11:00"]);
    timed_task(&mut st, "", &["01.01.2026 10:30 — 01.01.2026 11:30"]);

    let (_, totals) = st.query(&day_filter());
    assert_eq!(totals.time_label, "03:00:00", "сумма по задачам");
    assert_eq!(totals.actual_label, "01:30:00", "фактическое время: 10:00–11:30");
}

#[test]
fn actual_time_of_disjoint_and_adjacent_tasks_equals_sum() {
    let mut st = Store::open(temp_db("disjoint"));
    timed_task(&mut st, "", &["01.01.2026 09:00 — 01.01.2026 10:00"]);
    timed_task(&mut st, "", &["01.01.2026 10:00 — 01.01.2026 11:00"]);
    timed_task(&mut st, "", &["01.01.2026 12:00 — 01.01.2026 12:30"]);

    let (_, totals) = st.query(&day_filter());
    assert_eq!(totals.time_label, "02:30:00");
    assert_eq!(totals.actual_label, "02:30:00");
}

#[test]
fn actual_time_is_computed_inside_each_tag_group() {
    let mut st = Store::open(temp_db("by_tag"));
    timed_task(&mut st, "Альфа", &["01.01.2026 10:00 — 01.01.2026 11:00"]);
    timed_task(&mut st, "Альфа", &["01.01.2026 10:00 — 01.01.2026 11:00"]);
    timed_task(&mut st, "Бета", &["01.01.2026 10:00 — 01.01.2026 11:00"]);

    let (_, totals) = st.query(&day_filter());
    let alpha = totals.by_tag.iter().find(|t| t.name == "Альфа").unwrap();
    assert_eq!((alpha.time_label.as_str(), alpha.actual_label.as_str()), ("02:00:00", "01:00:00"));
    let beta = totals.by_tag.iter().find(|t| t.name == "Бета").unwrap();
    assert_eq!((beta.time_label.as_str(), beta.actual_label.as_str()), ("01:00:00", "01:00:00"));
    // Общий итог объединяет все три задачи, а не складывает группы.
    assert_eq!(totals.actual_label, "01:00:00");
}

#[test]
fn actual_time_counts_a_running_task_up_to_now() {
    let mut st = Store::open(temp_db("running"));
    let id = st.create_task(draft("A", "", "")).unwrap().task_id;
    let task = st.tasks.iter().find(|t| t.task_id == id).unwrap();
    let later = now_naive() + chrono::Duration::hours(2);
    let running = [task];
    let sums = tracker_core::logic::time_sums(&running, later);
    assert!(sums.sum >= 2.0 * 3600.0);
    assert_eq!(sums.sum, sums.actual);
}

#[test]
fn report_and_stats_show_both_sums() {
    let mut st = Store::open(temp_db("report"));
    timed_task(&mut st, "Альфа", &["01.01.2026 10:00 — 01.01.2026 11:00"]);
    timed_task(&mut st, "Альфа", &["01.01.2026 10:00 — 01.01.2026 11:00"]);

    let preview = tracker_core::report::report_preview(&st.tasks, &day_filter());
    assert_eq!((preview.total_label.as_str(), preview.actual_label.as_str()), ("02:00:00", "01:00:00"));
    assert_eq!(preview.by_tag[0].actual_label, "01:00:00");

    let stats = tracker_core::report::statistics(&st.tasks, &day_filter());
    assert_eq!((stats.total_label.as_str(), stats.actual_label.as_str()), ("02:00:00", "01:00:00"));
    assert_eq!(stats.avg_label, "01:00:00");
    assert_eq!(stats.by_user[0].total_label, "02:00:00");
    assert_eq!(stats.by_user[0].actual_label, "01:00:00");

    // Файлы всех форматов содержат обе подписи.
    let dir = std::env::temp_dir().join(format!("tt_report_files_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    for format in ["txt", "md", "xlsx", "csv"] {
        let path = dir.join(format!("stats.{format}"));
        tracker_core::report::write_stats_to(&path, format, &st.tasks, &day_filter(), "01.01.2026", "01.01.2026", "#4caf50").unwrap();
        assert!(path.exists(), "{format}");
    }
    for format in ["txt", "md"] {
        let path = dir.join(format!("stats.{format}"));
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("Сумма по задачам") && text.contains("Фактическое время"), "{format}");
    }
}

#[test]
fn removing_a_tag_strips_it_from_tasks() {
    let mut st = Store::open(temp_db("tag_cascade"));
    let a = st.create_task(draft("A", "Альфа", "")).unwrap().task_id;
    let b = st.create_task(draft("A", "Бета", "")).unwrap().task_id;
    let alpha_id = st.tags.iter().find(|t| t.name == "Альфа").unwrap().id;

    st.edit_dictionary(DictKind::Tag, DictAction::Remove { id: Some(alpha_id), name: None }).unwrap();
    let task = |id: &str| st.tasks.iter().find(|t| t.task_id == id).unwrap();
    assert!(task(&a).tags.is_empty(), "тег снят с задачи");
    assert_eq!(task(&b).tags, vec!["Бета"], "остальные теги не тронуты");

    // Отчёт и итоги больше не видят удалённый тег.
    let (_, totals) = st.query(&tracker_core::TaskFilter::default());
    assert!(totals.by_tag.iter().all(|t| t.name != "Альфа"));

    st.edit_dictionary(DictKind::Tag, DictAction::Clear).unwrap();
    assert!(st.tasks.iter().all(|t| t.tags.is_empty()));
}

#[test]
fn table_is_ordered_by_mode_then_first_start() {
    let mut st = Store::open(temp_db("table_order"));
    let done_old = timed_task(&mut st, "", &["01.01.2026 08:00 — 01.01.2026 09:00"]);
    let done_new = timed_task(&mut st, "", &["01.01.2026 12:00 — 01.01.2026 13:00"]);
    st.complete_task(&done_old).unwrap();
    st.complete_task(&done_new).unwrap();
    let paused_old = timed_task(&mut st, "", &["01.01.2026 07:00 — 01.01.2026 08:00"]);
    // Текущий (последний) диапазон начался позже всех, но первое начало самое раннее — на порядок влияет оно.
    let paused_new = timed_task(
        &mut st,
        "",
        &["01.01.2026 06:00 — 01.01.2026 06:30", "01.01.2026 14:00 — 01.01.2026 15:00"],
    );
    let running = timed_task(&mut st, "", &["01.01.2026 05:00 —"]);

    let (rows, _) = st.query(&day_filter());
    let ids: Vec<String> = rows
        .into_iter()
        .filter_map(|r| match r {
            tracker_core::ViewRow::Task(t) => Some(t.task_id),
            _ => None,
        })
        .collect();
    assert_eq!(ids, [running, paused_old, paused_new, done_new, done_old]);
}

#[test]
fn table_dates_span_all_ranges_but_editor_dates_are_the_current_range() {
    let mut st = Store::open(temp_db("current_range"));
    let id = timed_task(
        &mut st,
        "",
        &["01.01.2026 09:00 — 01.01.2026 10:00", "01.01.2026 12:00 — 01.01.2026 13:30"],
    );
    let view = tracker_core::models::AppTask::from_task(
        st.tasks.iter().find(|t| t.task_id == id).unwrap(),
        now_naive(),
    );
    // Таблица: самое первое начало и последнее завершение.
    assert_eq!(view.start_label, "01.01.2026 09:00");
    assert_eq!(view.end_label.as_deref(), Some("01.01.2026 13:30"));
    // Окно редактирования: текущий (последний) диапазон.
    assert_eq!(view.current_start, "01.01.2026 12:00");
    assert_eq!(view.current_end.as_deref(), Some("01.01.2026 13:30"));

    // Правка полей меняет именно текущий диапазон, первый остаётся прежним.
    st.set_task_dates(&id, "01.01.2026 12:30".into(), "01.01.2026 14:00".into()).unwrap();
    let task = st.tasks.iter().find(|t| t.task_id == id).unwrap();
    assert_eq!(task.ranges_str(), "01.01.2026 09:00 — 01.01.2026 10:00 | 01.01.2026 12:30 — 01.01.2026 14:00");

    // Идущая задача: завершения нет ни в таблице, ни в редакторе.
    let running = timed_task(&mut st, "", &["01.01.2026 08:00 — 01.01.2026 08:30", "01.01.2026 11:00 —"]);
    let view = tracker_core::models::AppTask::from_task(
        st.tasks.iter().find(|t| t.task_id == running).unwrap(),
        now_naive(),
    );
    assert_eq!(view.start_label, "01.01.2026 08:00");
    assert_eq!(view.end_label, None);
    assert_eq!((view.current_start.as_str(), view.current_end), ("01.01.2026 11:00", None));
}
