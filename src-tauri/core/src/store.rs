//! Хранилище: загрузка/сохранение JSON-файла (совместимо с Python),
//! CRUD задач, пользователей, тегов, клиентов. Каждая мутация сохраняет файл.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::dt::{now_naive, parse_dt};
use crate::logic::{build_rows, build_totals, apply_filter, TaskFilter, TaskIdGen, Totals, ViewRow};
use crate::models::{AppState, AppTask, ColumnPref, Entity, Interval, Settings, StatusDef, Task, TaskDraft, TaskStatus};

/// Действие таймера над задачей.
#[derive(Clone, Copy, Debug, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskAction {
    Start,
    Pause,
    Resume,
    Complete,
}

#[derive(Debug)]
pub struct Store {
    pub path: PathBuf,
    pub settings: Settings,
    pub users: Vec<String>,
    pub tags: Vec<Entity>,
    pub clients: Vec<Entity>,
    pub statuses: Vec<StatusDef>,
    pub tasks: Vec<Task>,
    pub next_tag_id: u64,
    pub next_client_id: u64,
    pub next_status_id: u64,
    id_gen: TaskIdGen,
}

impl Store {
    /// Открыть файл. Не существует — пустая база. Битый — переносится в
    /// «.corrupt.bak», старт с пустой базой (как Python).
    pub fn open(path: PathBuf) -> Store {
        let mut st = Store {
            path,
            settings: Settings::default(),
            users: Vec::new(),
            tags: Vec::new(),
            clients: Vec::new(),
            statuses: Vec::new(),
            tasks: Vec::new(),
            next_tag_id: 1,
            next_client_id: 1,
            next_status_id: 1,
            id_gen: TaskIdGen::new(),
        };
        st.load();
        st
    }

    fn load(&mut self) {
        if !self.path.exists() {
            return;
        }
        let raw = match fs::read_to_string(&self.path) {
            Ok(s) => s,
            Err(_) => {
                self.move_corrupt();
                return;
            }
        };
        let val: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => {
                self.move_corrupt();
                return;
            }
        };

        self.settings = Settings::from_json(val.get("settings").unwrap_or(&serde_json::Value::Null));

        let users_arr = val.get("users").and_then(|x| x.as_array());
        self.users = users_arr
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        if let Some(arr) = val.get("tags").and_then(|x| x.as_array()) {
            self.tags = arr.iter().filter_map(Entity::from_json).collect();
        }
        if let Some(arr) = val.get("clients").and_then(|x| x.as_array()) {
            self.clients = arr.iter().filter_map(Entity::from_json).collect();
        }
        if let Some(arr) = val.get("statuses").and_then(|x| x.as_array()) {
            self.statuses = arr.iter().filter_map(StatusDef::from_json).collect();
        }
        let load_now = now_naive();
        if let Some(arr) = val.get("tasks").and_then(|x| x.as_array()) {
            self.tasks = arr
                .iter()
                .filter_map(|v| Task::from_json(v, load_now))
                .collect();
        }
        self.next_tag_id = val
            .get("_next_tag_id")
            .and_then(|x| x.as_u64())
            .unwrap_or(1);
        self.next_client_id = val
            .get("_next_client_id")
            .and_then(|x| x.as_u64())
            .unwrap_or(1);
        self.next_status_id = val
            .get("_next_status_id")
            .and_then(|x| x.as_u64())
            .unwrap_or(1);

        // Автодополнение пользователей из задач (миграция).
        let mut known: std::collections::HashSet<String> = self.users.iter().cloned().collect();
        for t in &self.tasks {
            if !t.user.is_empty() && !known.contains(&t.user) {
                known.insert(t.user.clone());
                self.users.push(t.user.clone());
            }
        }
        if !self.settings.username.is_empty() && !known.contains(&self.settings.username) {
            self.users.push(self.settings.username.clone());
        }

        let ids: Vec<String> = self.tasks.iter().map(|t| t.task_id.clone()).collect();
        self.id_gen.warm(&ids);
    }

    fn move_corrupt(&mut self) {
        if self.path.exists() {
            let backup = self.path.with_extension("corrupt.bak");
            let _ = fs::remove_file(&backup);
            let _ = fs::rename(&self.path, &backup);
        }
    }

    /// Атомарная запись: во временный файл в том же каталоге, затем rename.
    /// Любая ошибка возвращается вызывающему (и доходит до интерфейса).
    pub fn save(&self) -> Result<(), String> {
        let dir = self.path.parent().unwrap_or(&self.path);
        fs::create_dir_all(dir)
            .map_err(|e| format!("Не удалось создать каталог данных «{}»: {e}", dir.display()))?;
        let now = now_naive();
        let obj = serde_json::json!({
            "settings": self.settings.to_json(),
            "users": self.users,
            "tags": self.tags,
            "clients": self.clients,
            "statuses": self.statuses,
            "tasks": self.tasks.iter().map(|t| t.to_json(now)).collect::<Vec<_>>(),
            "_next_tag_id": self.next_tag_id,
            "_next_client_id": self.next_client_id,
            "_next_status_id": self.next_status_id,
        });
        let pretty = serde_json::to_string_pretty(&obj)
            .map_err(|e| format!("Не удалось сериализовать данные: {e}"))?;
        let tmp = self.path.with_extension("json.tmp");
        let write = || -> std::io::Result<()> {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(pretty.as_bytes())?;
            f.sync_all()?;
            drop(f);
            fs::rename(&tmp, &self.path)
        };
        write().map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("Не удалось сохранить данные в «{}»: {e}", self.path.display())
        })
    }

    /// Идущие задачи и сколько секунд длится их текущий (открытый) диапазон.
    pub fn running_elapsed(&self, now: chrono::NaiveDateTime) -> Vec<(&Task, i64)> {
        self.tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Running)
            .filter_map(|t| {
                let open = t.intervals.iter().rev().find(|i| i.stop.is_none())?;
                Some((t, (now - open.start).num_seconds().max(0)))
            })
            .collect()
    }

    // -----------------------------------------------------------------
    // Представления
    // -----------------------------------------------------------------

    pub fn state(&self) -> AppState {
        AppState {
            settings: self.settings.clone(),
            users: self.users.clone(),
            tags: self.tags.clone(),
            clients: self.clients.clone(),
            statuses: self.statuses.clone(),
        }
    }

    pub fn query(&self, filter: &TaskFilter) -> (Vec<ViewRow>, Totals) {
        let now = now_naive();
        let filtered = apply_filter(&self.tasks, filter, now);
        let rows = build_rows(&filtered, &self.settings.grouping);
        let totals = build_totals(&filtered);
        (rows, totals)
    }

    // -----------------------------------------------------------------
    // Задачи
    // -----------------------------------------------------------------

    pub fn create_task(&mut self, draft: TaskDraft) -> Result<AppTask, String> {
        let now = now_naive();
        let mut task = Task {
            task_id: self.id_gen.next_id(),
            user: String::new(),
            orders: Vec::new(),
            tags: Vec::new(),
            client: String::new(),
            status: crate::models::TaskStatus::Running,
            custom_status: String::new(),
            comment: String::new(),
            our_car: false,
            intervals: vec![crate::models::Interval {
                start: now,
                stop: None,
            }],
        };
        draft.apply(&mut task);
        self.ensure_user(&task.user);
        for tag in task.tags.clone() {
            self.ensure_tag(&tag);
        }
        self.ensure_client(&task.client);
        self.ensure_status(&task.custom_status);
        let view = AppTask::from_task(&task, now_naive());
        self.tasks.push(task);
        self.save()?;
        Ok(view)
    }

    pub fn update_task(&mut self, task_id: &str, draft: TaskDraft) -> Result<(), String> {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            draft.apply(t);
        }
        self.ensure_user(&draft.user);
        for tag in draft.tags.clone() {
            let tag = tag.trim().to_string();
            if !tag.is_empty() {
                self.ensure_tag(&tag);
            }
        }
        self.ensure_client(&draft.client);
        self.ensure_status(&draft.custom_status);
        self.save()
    }

    /// Сменить пользовательский статус задачи (каскадно в справочник, если
    /// статус ещё не существует).
    pub fn set_task_status(&mut self, task_id: &str, status: String) -> Result<(), String> {
        let status = status.trim().to_string();
        self.ensure_status(&status);
        if let Some(t) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            t.custom_status = status;
        }
        self.save()
    }

    /// Обновить даты ПОСЛЕДНЕГО диапазона (поля «Начало/Завершение» редактора).
    pub fn set_task_dates(&mut self, task_id: &str, start: String, end: String) -> Result<(), String> {
        let start_dt = parse_dt(&start)
            .ok_or_else(|| "Неверный формат начала (дд.мм.гггг чч:мм)".to_string())?;
        let end_dt = if end.trim().is_empty() {
            None
        } else {
            Some(parse_dt(&end).ok_or_else(|| "Неверный формат завершения (дд.мм.гггг чч:мм)".to_string())?)
        };
        if let Some(e) = end_dt {
            if e < start_dt {
                return Err("Завершение раньше начала".to_string());
            }
        }
        if let Some(t) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            t.set_last_interval(start_dt, end_dt);
        }
        self.save()
    }

    /// Заменить все диапазоны задачи строками «дд.мм.гггг чч:мм — …»
    /// (завершение может отсутствовать = открытый диапазон).
    pub fn set_task_intervals(&mut self, task_id: &str, lines: Vec<String>) -> Result<(), String> {
        let non_empty: Vec<&str> = lines
            .iter()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if non_empty.is_empty() {
            return Err("Должен быть хотя бы один диапазон".to_string());
        }
        let mut intervals: Vec<Interval> = Vec::new();
        for line in non_empty {
            let iv = Self::parse_interval_line(line)?;
            if let Some(prev) = intervals.last().and_then(|i| i.stop) {
                if iv.start < prev {
                    return Err("Диапазоны должны идти по порядку времени".to_string());
                }
            }
            intervals.push(iv);
        }
        if let Some(open_idx) = intervals.iter().position(|i| i.stop.is_none()) {
            if open_idx != intervals.len() - 1 {
                return Err("Открытым может быть только последний диапазон".to_string());
            }
        }
        if let Some(t) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            let open = intervals.last().map(|i| i.stop.is_none()).unwrap_or(false);
            t.intervals = intervals;
            t.status = if open {
                TaskStatus::Running
            } else if t.status == TaskStatus::Running {
                TaskStatus::Paused
            } else {
                t.status.clone()
            };
        }
        self.save()
    }

    /// Строка «дд.мм.гггг чч:мм [»— …»]»; завершение может отсутствовать,
    /// быть «открыт», датой/временем.
    fn parse_interval_line(line: &str) -> Result<Interval, String> {
        let line = line
            .replace("—", "\u{0}")
            .replace("–", "\u{0}")
            .replace('-', "\u{0}");
        let mut parts = line.split('\u{0}');
        let start_text = parts.next().unwrap_or("").trim();
        let end_text = parts.next().unwrap_or("").trim();
        let start = parse_dt(start_text)
            .ok_or_else(|| format!("Неверный формат начала: «{}»", start_text))?;
        let stop = if end_text.is_empty() || end_text.eq_ignore_ascii_case("открыт") {
            None
        } else {
            let dt = parse_dt(end_text).or_else(|| {
                chrono::NaiveTime::parse_from_str(end_text, "%H:%M")
                    .ok()
                    .map(|t| start.date().and_time(t))
            });
            Some(
                dt.ok_or_else(|| format!("Неверный формат завершения: «{}»", end_text))?,
            )
        };
        Ok(Interval { start, stop })
    }

    /// Применить действие таймера к задаче и сохранить файл.
    pub fn task_action(&mut self, task_id: &str, action: TaskAction) -> Result<(), String> {
        let now = now_naive();
        if let Some(t) = self.tasks.iter_mut().find(|t| t.task_id == task_id) {
            match action {
                TaskAction::Start | TaskAction::Resume => {
                    if t.status != TaskStatus::Running {
                        t.start(now);
                    }
                }
                TaskAction::Pause => t.pause(now),
                TaskAction::Complete => t.complete(now),
            }
        }
        self.save()
    }

    pub fn start_task(&mut self, task_id: &str) -> Result<(), String> {
        self.task_action(task_id, TaskAction::Start)
    }

    pub fn pause_task(&mut self, task_id: &str) -> Result<(), String> {
        self.task_action(task_id, TaskAction::Pause)
    }

    pub fn resume_task(&mut self, task_id: &str) -> Result<(), String> {
        self.task_action(task_id, TaskAction::Resume)
    }

    pub fn complete_task(&mut self, task_id: &str) -> Result<(), String> {
        self.task_action(task_id, TaskAction::Complete)
    }

    pub fn remove_task(&mut self, task_id: &str) -> Result<(), String> {
        self.remove_tasks(&[task_id.to_string()])
    }

    /// Удалить несколько задач одним сохранением файла.
    pub fn remove_tasks(&mut self, ids: &[String]) -> Result<(), String> {
        self.tasks.retain(|t| !ids.contains(&t.task_id));
        self.save()
    }

// -----------------------------------------------------------------
// Экспорт / импорт JSON (как Python Store.export_json / import_json)
// -----------------------------------------------------------------

pub fn export_json_file(
    st: &Store,
    path: &std::path::Path,
    export_tasks: bool,
    export_tags: bool,
    export_clients: bool,
    export_users: bool,
    export_statuses: bool,
) -> Result<(), String> {
    let mut data = serde_json::Map::new();
    if export_users {
        data.insert("users".into(), serde_json::to_value(&st.users).unwrap_or_default());
    }
    if export_tags {
        data.insert(
            "tags".into(),
            serde_json::to_value(&st.tags).unwrap_or_default(),
        );
    }
    if export_clients {
        data.insert(
            "clients".into(),
            serde_json::to_value(&st.clients).unwrap_or_default(),
        );
    }
    if export_statuses {
        data.insert(
            "statuses".into(),
            serde_json::to_value(&st.statuses).unwrap_or_default(),
        );
    }
    if export_tasks {
        let now = now_naive();
        data.insert(
            "tasks".into(),
            serde_json::Value::Array(st.tasks.iter().map(|t| t.to_json(now)).collect()),
        );
    }
    let json = serde_json::Value::Object(data);
    let text = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn import_json_file(
    st: &mut Store,
    path: &std::path::Path,
    import_tasks: bool,
    import_tags: bool,
    import_clients: bool,
    import_users: bool,
    import_statuses: bool,
) -> Result<(), String> {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return Err(format!(
                "Не удалось открыть файл:\n{e}"
            ))
        }
    };
    let data: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!(
                "Файл не является корректным JSON (ошибка: {e}).\n\
                 Возможные причины:\n\
                 \u{2022} файл повреждён или был обрезан при записи;\n\
                 \u{2022} это не экспорт Time Tracker, а файл в другом формате;\n\
                 \u{2022} файл редактировался вручную и в нём осталась синтаксическая ошибка."
            ))
        }
    };

    if !data.is_object() {
        return Err(
            "Содержимое файла должно быть JSON-объектом (со скобками {{ }}),\n\
             а не списком. Возможно, это не экспорт Time Tracker."
                .to_string(),
        );
    }
    let known = ["tasks", "tags", "clients", "users", "statuses", "settings"];
    if !known.iter().any(|k| data.get(k).is_some()) {
        return Err(
            "В выбранном файле не найдено разделов данных\n\
             (tasks, tags, clients, users, statuses).\n\
             Возможные причины:\n\
             \u{2022} выбран не тот файл;\n\
             \u{2022} это не экспорт Time Tracker."
                .to_string(),
        );
    }
    for key in ["tasks", "tags", "clients", "users", "statuses"] {
        if let Some(v) = data.get(key) {
            if !v.is_array() {
                return Err(format!(
                    "Раздел «{key}» должен быть списком. Файл повреждён или изменён вручную."
                ));
            }
        }
    }

    if import_tags {
        if let Some(arr) = data.get("tags").and_then(|x| x.as_array()) {
            for td in arr {
                let name = td.get("name").and_then(|x| x.as_str()).unwrap_or("");
                if !name.is_empty() && !st.tags.iter().any(|t| t.name == name) {
                    st.add_tag(name)?;
                }
            }
        }
    }

    if import_clients {
        if let Some(arr) = data.get("clients").and_then(|x| x.as_array()) {
            for cd in arr {
                let name = cd.get("name").and_then(|x| x.as_str()).unwrap_or("");
                if !name.is_empty() && !st.clients.iter().any(|c| c.name == name) {
                    st.add_client(name)?;
                }
            }
        }
    }

    if import_users {
        if let Some(arr) = data.get("users").and_then(|x| x.as_array()) {
            for u in arr {
                if let Some(name) = u.as_str() {
                    if !name.trim().is_empty() {
                        st.add_user(name)?;
                    }
                }
            }
        }
    }

    if import_statuses {
        if let Some(arr) = data.get("statuses").and_then(|x| x.as_array()) {
            for sd in arr {
                if let Some(s) = StatusDef::from_json(sd) {
                    st.add_status(&s.name, &s.color)?;
                }
            }
        }
    }

    if import_tasks {
        let mut existing: std::collections::HashSet<String> =
            st.tasks.iter().map(|t| t.task_id.clone()).collect();
        if let Some(arr) = data.get("tasks").and_then(|x| x.as_array()) {
            let now = now_naive();
            for td in arr {
                if !td.is_object() {
                    return Err(
                        "Задача в файле не является объектом JSON (ожидалось {{…}}).\n\
                         Возможные причины:\n\
                         \u{2022} файл редактировался вручную и была сдвинута структура;\n\
                         \u{2022} это не экспорт Time Tracker."
                            .to_string(),
                    );
                }
                let tid = td.get("task_id").and_then(|x| x.as_str()).unwrap_or("");
                if tid.is_empty() || existing.contains(tid) {
                    continue;
                }
                match Task::from_json(td, now) {
                    Some(mut task) => {
                        task.status = crate::models::TaskStatus::Completed;
                        for seg in task.intervals.iter_mut() {
                            if seg.stop.is_none() {
                                seg.stop = Some(seg.start);
                            }
                        }
                        existing.insert(task.task_id.clone());
                        st.tasks.push(task);
                    }
                    None => {
                        return Err(format!(
                            "Задача «{tid}»: неверный формат данных.\n\
                             Возможные причины:\n\
                             \u{2022} вручную изменены или удалены поля задачи;\n\
                             \u{2022} неверный формат дат (нужен «дд.мм.гггг ЧЧ:ММ»);\n\
                             \u{2022} задача создана не в этой версии программы."
                        ))
                    }
                }
            }
            let ids: Vec<String> = st.tasks.iter().map(|t| t.task_id.clone()).collect();
            st.id_gen.warm(&ids);
        }
    }

    // Автодополнение пользователей из импортированных задач (как при загрузке).
    let mut known: std::collections::HashSet<String> = st.users.iter().cloned().collect();
    for t in &st.tasks {
        if !t.user.is_empty() && !known.contains(&t.user) {
            known.insert(t.user.clone());
            st.users.push(t.user.clone());
        }
    }

    st.save()
}
// -----------------------------------------------------------------
// Настройки
// -----------------------------------------------------------------

    pub fn set_settings(&mut self, username: String, grouping: String, accent_color: String) -> Result<(), String> {
        self.settings.username = username.trim().to_string();
        if ["none", "day", "client"].contains(&grouping.as_str()) {
            self.settings.grouping = grouping;
        }
        let accent = accent_color.trim().to_string();
        if let Some(hex) = normalize_hex(&accent) {
            self.settings.accent_color = hex;
        }
        let name = self.settings.username.clone();
        if !name.is_empty() {
            self.ensure_user(&name);
        }
        self.save()
    }

    /// Сохранить цвет ячейки «Наша машина».
    pub fn set_our_car_color(&mut self, color: String) -> Result<(), String> {
        let color = color.trim().to_string();
        if let Some(hex) = normalize_hex(&color) {
            self.settings.our_car_color = hex;
        }
        self.save()
    }

    /// Сохранить порядок и видимость колонок таблицы.
    pub fn set_columns(&mut self, columns: Vec<ColumnPref>) -> Result<(), String> {
        self.settings.columns = columns;
        self.save()
    }

    /// Параметры трея и напоминаний.
    pub fn set_tray_prefs(&mut self, minimize_to_tray: bool, reminder_hours: u32) -> Result<(), String> {
        self.settings.minimize_to_tray = minimize_to_tray;
        self.settings.reminder_hours = reminder_hours.min(crate::models::MAX_REMINDER_HOURS);
        self.save()
    }

    /// Сохранить пользовательский каталог бэкапов ("" = по умолчанию).
    pub fn set_backups_dir(&mut self, dir: String) -> Result<(), String> {
        let dir = dir.trim().to_string();
        if dir.chars().count() > 512 {
            return Err("Путь к каталогу бэкапов слишком длинный (макс. 512 символов)".to_string());
        }
        self.settings.backup_dir = dir;
        self.save()
    }
}

// -----------------------------------------------------------------
// Пути (вычисление снаружи)
// -----------------------------------------------------------------

/// Нормализовать цвет из настроек в "#rrggbb" (или None, если некорректен).
fn normalize_hex(s: &str) -> Option<String> {
    let t = s.trim();
    let digits = t.strip_prefix('#').unwrap_or(t);
    if digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(format!("#{}", digits.to_ascii_lowercase()))
    } else {
        None
    }
}

/// Переместить элемент с индексом `from` на индекс `to`; `false`, если границы неверны.
pub(crate) fn shift_vec<T>(v: &mut Vec<T>, from: usize, to: usize) -> bool {
    if from >= v.len() || to >= v.len() || from == to {
        return false;
    }
    let item = v.remove(from);
    v.insert(to, item);
    true
}

pub fn data_file(base_dir: std::path::PathBuf) -> PathBuf {
    base_dir.join("data").join("time_tracker_v2_data.json")
}

pub fn legacy_data_file(base_dir: std::path::PathBuf) -> PathBuf {
    base_dir.join("time_tracker_v2_data.json")
}