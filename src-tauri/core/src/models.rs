//! Модели данных, совместимые с data/time_tracker_v2_data.json
//! (Python-версия). Загрузка «ленивая» — неверные записи пропускаются
//! точно так же, как в оригинале.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value as J;

use crate::dt::{fmt_dt, now_naive, parse_dt};

// ---------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColumnPref {
    pub key: String,
    pub visible: bool,
}

impl ColumnPref {
    pub fn from_json(v: &J) -> Option<ColumnPref> {
        let obj = v.as_object()?;
        let key = obj.get("key")?.as_str()?.trim();
        if key.is_empty() {
            return None;
        }
        Some(ColumnPref {
            key: key.to_string(),
            visible: obj.get("visible").and_then(|x| x.as_bool()).unwrap_or(true),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub username: String,
    pub grouping: String, // "none" | "day" | "client"
    pub accent_color: String, // цвет кнопок/заголовков (hex "#rrggbb")
    pub our_car_color: String, // цвет ячейки «Наша машина» (hex "#rrggbb")
    pub font_family: String, // вид шрифта таблицы задач (CSS font-family)
    pub font_size: u32, // размер шрифта таблицы задач (px)
    pub columns: Vec<ColumnPref>, // порядок и видимость колонок таблицы
    pub backup_dir: String, // пользовательский каталог бэкапов ("") = по умолчанию
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            username: String::new(),
            grouping: "none".into(),
            accent_color: "#4caf50".into(),
            our_car_color: "#2196f3".into(),
            font_family: "Roboto".into(),
            font_size: 14,
            columns: Vec::new(),
            backup_dir: String::new(),
        }
    }
}

impl Settings {
    pub fn from_json(v: &J) -> Settings {
        let obj = v.as_object();
        let grouping = obj
            .and_then(|o| o.get("grouping"))
            .and_then(|x| x.as_str())
            .unwrap_or("none")
            .to_string();
        let accent_color = obj
            .and_then(|o| o.get("accent_color"))
            .and_then(|x| x.as_str())
            .unwrap_or("#4caf50")
            .to_string();
        let our_car_color = obj
            .and_then(|o| o.get("our_car_color"))
            .and_then(|x| x.as_str())
            .unwrap_or("#2196f3")
            .to_string();
        let font_family = obj
            .and_then(|o| o.get("font_family"))
            .and_then(|x| x.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Roboto".to_string());
        let font_size = obj
            .and_then(|o| o.get("font_size"))
            .and_then(|x| x.as_u64())
            .filter(|v| *v >= 8 && *v <= 40)
            .unwrap_or(14) as u32;
        let columns = obj
            .and_then(|o| o.get("columns"))
            .and_then(|x| x.as_array())
            .map(|a| a.iter().filter_map(ColumnPref::from_json).collect())
            .unwrap_or_default();
        let backup_dir = obj
            .and_then(|o| o.get("backup_dir"))
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        Settings {
            username: obj
                .and_then(|o| o.get("username"))
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string(),
            grouping,
            accent_color,
            our_car_color,
            font_family,
            font_size,
            columns,
            backup_dir,
        }
    }

    pub fn to_json(&self) -> J {
        serde_json::json!({
            "username": self.username,
            "grouping": self.grouping,
            "accent_color": self.accent_color,
            "our_car_color": self.our_car_color,
            "font_family": self.font_family,
            "font_size": self.font_size,
            "columns": self.columns,
            "backup_dir": self.backup_dir,
        })
    }
}

// ---------------------------------------------------------------------
// Entity (тег / клиент)
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    pub id: u64,
    pub name: String,
}

impl Entity {
    pub fn from_json(v: &J) -> Option<Entity> {
        let obj = v.as_object()?;
        let name = obj.get("name")?.as_str()?.trim();
        if name.is_empty() {
            return None;
        }
        let id = obj.get("id").and_then(|x| x.as_u64()).unwrap_or(0);
        Some(Entity {
            id,
            name: name.to_string(),
        })
    }
}

// ---------------------------------------------------------------------
// StatusDef (пользовательский статус задачи: имя + цвет ячейки)
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusDef {
    pub id: u64,
    pub name: String,
    pub color: String,
}

impl StatusDef {
    pub fn from_json(v: &J) -> Option<StatusDef> {
        let obj = v.as_object()?;
        let name = obj.get("name")?.as_str()?.trim();
        if name.is_empty() {
            return None;
        }
        let id = obj.get("id").and_then(|x| x.as_u64()).unwrap_or(0);
        let color = obj.get("color").and_then(|x| x.as_str()).unwrap_or("default").to_string();
        Some(StatusDef {
            id,
            name: name.to_string(),
            color,
        })
    }
}

// ---------------------------------------------------------------------
// Task / Interval / Status
// ---------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    Running,
    Paused,
    Completed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Running => "running",
            TaskStatus::Paused => "paused",
            TaskStatus::Completed => "completed",
        }
    }

    pub fn from_str(s: &str) -> TaskStatus {
        match s {
            "running" => TaskStatus::Running,
            "paused" => TaskStatus::Paused,
            _ => TaskStatus::Completed,
        }
    }
}

impl Serialize for TaskStatus {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct Interval {
    pub start: NaiveDateTime,
    pub stop: Option<NaiveDateTime>,
}

fn parse_seg(v: &J) -> Option<Interval> {
    let obj = v.as_object()?;
    let start = obj.get("start").and_then(|x| x.as_str()).and_then(parse_dt)?;
    let stop = obj
        .get("stop")
        .and_then(|x| x.as_str())
        .and_then(parse_dt);
    Some(Interval { start, stop })
}

#[derive(Clone, Debug)]
pub struct Task {
    pub task_id: String,
    pub user: String,
    pub order: String,
    pub tags: Vec<String>,
    pub client: String,
    pub status: TaskStatus,
    pub custom_status: String,
    pub comment: String,
    pub intervals: Vec<Interval>,
    pub our_car: bool,
}

fn str_at(obj: &serde_json::Map<String, J>, key: &str) -> String {
    obj.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string()
}

fn tags_from_json(obj: &serde_json::Map<String, J>) -> Vec<String> {
    if let Some(arr) = obj.get("tags").and_then(|x| x.as_array()) {
        let mut v: Vec<String> = arr
            .iter()
            .filter_map(|x| x.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        v.dedup();
        if !v.is_empty() {
            return v;
        }
    }
    // Совместимость со старым форматом: одно поле «tag» (строка).
    let old = str_at(obj, "tag");
    if old.is_empty() {
        Vec::new()
    } else {
        vec![old]
    }
}

impl Task {
    pub fn from_json(v: &J, load_now: NaiveDateTime) -> Option<Task> {
        let obj = v.as_object()?;
        let mut intervals: Vec<Interval> = Vec::new();
        if let Some(arr) = obj.get("intervals").and_then(|x| x.as_array()) {
            for seg in arr {
                if let Some(iv) = parse_seg(seg) {
                    intervals.push(iv);
                }
            }
        }
        if intervals.is_empty() {
            // Поведение Python: если диапазоны не разобрались — берём
            // верхнеуровневые start/end, иначе даже рукотворный now/now.
            let st = str_at(obj, "start");
            let st = parse_dt(&st).unwrap_or(load_now);
            let sp = parse_dt(&str_at(obj, "end")).unwrap_or(st);
            intervals.push(Interval {
                start: st,
                stop: Some(sp),
            });
        }
        Some(Task {
            task_id: str_at(obj, "task_id"),
            user: str_at(obj, "user"),
            order: str_at(obj, "order"),
            tags: tags_from_json(obj),
            client: str_at(obj, "client"),
            status: TaskStatus::from_str(&str_at(obj, "status")),
            custom_status: str_at(obj, "custom_status"),
            comment: str_at(obj, "comment"),
            intervals,
            our_car: obj.get("our_car").and_then(|x| x.as_bool()).unwrap_or(false),
        })
    }

    pub fn start_date(&self) -> NaiveDateTime {
        self.intervals[0].start
    }

    pub fn end(&self) -> Option<NaiveDateTime> {
        self.intervals.last().and_then(|i| i.stop)
    }

    pub fn is_open(&self) -> bool {
        self.intervals.last().map(|i| i.stop.is_none()).unwrap_or(false)
    }

    pub fn start_str(&self) -> String {
        fmt_dt(self.start_date())
    }

    pub fn end_str(&self) -> String {
        self.end().map(fmt_dt).unwrap_or_default()
    }

    /// Строка всех диапазонов «дд.мм.гггг ЧЧ:ММ — … | …» (как Python).
    pub fn ranges_str(&self) -> String {
        let mut parts = Vec::new();
        for seg in &self.intervals {
            parts.push(match seg.stop {
                Some(stop) => format!("{} — {}", fmt_dt(seg.start), fmt_dt(stop)),
                None => format!("{} — открыт", fmt_dt(seg.start)),
            });
        }
        parts.join(" | ")
    }

    /// Суммарное время в секундах, включая открытый интервал (как Python).
    pub fn total_seconds(&self, now: NaiveDateTime) -> f64 {
        let mut total = 0.0_f64;
        for seg in &self.intervals {
            if let Some(stop) = seg.stop {
                total += (stop - seg.start).num_seconds() as f64;
            }
        }
        if let Some(last) = self.intervals.last() {
            if last.stop.is_none() {
                total += (now - last.start).num_seconds().max(0) as f64;
            }
        }
        total.max(0.0)
    }

    /// Включить (старт/возобновление): открывает новый интервал.
    pub fn start(&mut self, at: NaiveDateTime) {
        self.intervals.push(Interval {
            start: at,
            stop: None,
        });
        self.status = TaskStatus::Running;
    }

    /// Пауза: закрывает открытый интервал.
    pub fn pause(&mut self, at: NaiveDateTime) {
        if let Some(last) = self.intervals.last_mut() {
            if last.stop.is_none() {
                last.stop = Some(at);
            }
        }
        self.status = TaskStatus::Paused;
    }

    /// Завершить: закрывает открытый интервал.
    pub fn complete(&mut self, at: NaiveDateTime) {
        if let Some(last) = self.intervals.last_mut() {
            if last.stop.is_none() {
                last.stop = Some(at);
            }
        }
        self.status = TaskStatus::Completed;
    }

    /// Перезаписать даты ПОСЛЕДНЕГО диапазона (поля «Начало/Завершение» редактора).
    pub fn set_last_interval(&mut self, start: NaiveDateTime, stop: Option<NaiveDateTime>) {
        if let Some(last) = self.intervals.last_mut() {
            last.start = start;
            last.stop = stop;
        }
        self.status = match stop {
            None => TaskStatus::Running,
            Some(_) if self.status == TaskStatus::Running => TaskStatus::Paused,
            Some(_) => self.status.clone(),
        };
    }

    /// Сериализация в формат файла (поля и порядок как в Python).
    pub fn to_json(&self, now: NaiveDateTime) -> J {
        let intervals: Vec<J> = self
            .intervals
            .iter()
            .map(|i| {
                serde_json::json!({
                    "start": fmt_dt(i.start),
                    "stop": i.stop.map(fmt_dt),
                })
            })
            .collect();
        serde_json::json!({
            "task_id": self.task_id,
            "user": self.user,
            "order": self.order,
            "tags": self.tags,
            "tag": self.tags.first().cloned().unwrap_or_default(),
            "client": self.client,
            "status": self.status.as_str(),
            "custom_status": self.custom_status,
            "comment": self.comment,
            "our_car": self.our_car,
            "seconds": self.total_seconds(now),
            "intervals": intervals,
            "start": fmt_dt(self.start_date()),
            "end": self.end().map(fmt_dt),
        })
    }
}

// ---------------------------------------------------------------------
// Представление задачи для фронтенда
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTask {
    pub task_id: String,
    pub user: String,
    pub order: String,
    pub tags: Vec<String>,
    pub client: String,
    pub comment: String,
    #[serde(rename = "mode")]
    pub status: String,
    #[serde(rename = "status")]
    pub custom_status: String,
    pub seconds: f64,
    #[serde(rename = "time")]
    pub time_label: String,
    #[serde(rename = "start")]
    pub start_label: String,
    #[serde(rename = "end")]
    pub end_label: Option<String>,
    pub intervals_count: usize,
    pub ranges: Vec<AppInterval>,
    #[serde(rename = "ourCar")]
    pub our_car: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInterval {
    pub start: String,
    pub stop: Option<String>,
}

impl AppTask {
    pub fn from_task(t: &Task, now: NaiveDateTime) -> AppTask {
        AppTask {
            task_id: t.task_id.clone(),
            user: t.user.clone(),
            order: t.order.clone(),
            tags: t.tags.clone(),
            client: t.client.clone(),
            comment: t.comment.clone(),
            status: t.status.as_str().to_string(),
            custom_status: t.custom_status.clone(),
            seconds: t.total_seconds(now),
            time_label: crate::dt::fmt_td(t.total_seconds(now)),
            start_label: fmt_dt(t.start_date()),
            end_label: t.end().map(fmt_dt),
            intervals_count: t.intervals.len(),
            ranges: t
                .intervals
                .iter()
                .map(|i| AppInterval {
                    start: fmt_dt(i.start),
                    stop: i.stop.map(fmt_dt),
                })
                .collect(),
            our_car: t.our_car,
        }
    }
}

/// Полный срез состояния для интерфейса.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub settings: Settings,
    pub users: Vec<String>,
    pub tags: Vec<Entity>,
    pub clients: Vec<Entity>,
    pub statuses: Vec<StatusDef>,
    pub tasks: Vec<AppTask>,
}

// Типы, удобные для команд: создание и правка задачи.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDraft {
    pub user: String,
    pub order: String,
    pub tags: Vec<String>,
    pub client: String,
    pub comment: String,
    pub custom_status: String,
    #[serde(default)]
    pub our_car: bool,
}

impl TaskDraft {
    pub fn apply(&self, t: &mut Task) {
        t.user = self.user.trim().to_string();
        t.order = self.order.trim().to_string();
        let mut tags: Vec<String> = self
            .tags
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        tags.dedup();
        t.tags = tags;
        t.client = self.client.trim().to_string();
        t.comment = self.comment.trim().to_string();
        t.custom_status = self.custom_status.trim().to_string();
        t.our_car = self.our_car;
    }
}

pub fn load_now() -> NaiveDateTime {
    now_naive()
}