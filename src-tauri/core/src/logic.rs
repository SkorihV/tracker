//! Логика: генерация task_id, фильтры/поиск, группировка, итоги.
//! Повторяет поведение Python-версии.

use std::collections::HashMap;

use chrono::{NaiveDate, NaiveDateTime};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use crate::dt::{fmt_td, parse_date, today_key};
use crate::models::{AppTask, Task};

// ---------------------------------------------------------------------
// Генератор task_id: «ГГГГММДД-NNN-XXxx» (md5 первых 4 hex).
// ---------------------------------------------------------------------

#[derive(Default, Debug)]
pub struct TaskIdGen {
    counters: HashMap<String, u32>,
}

impl TaskIdGen {
    pub fn new() -> Self {
        TaskIdGen {
            counters: HashMap::new(),
        }
    }

    /// Восстановить счётчики по существующим ID (как Python).
    pub fn warm(&mut self, ids: &[String]) {
        for id in ids {
            let parts: Vec<&str> = id.split('-').collect();
            if parts.len() >= 2 {
                if let Ok(num) = parts[1].parse::<u32>() {
                    let key = parts[0].to_string();
                    let cur = self.counters.get(&key).copied().unwrap_or(0);
                    if num > cur {
                        self.counters.insert(key, num);
                    }
                }
            }
        }
    }

    pub fn next_id(&mut self) -> String {
        let key = today_key();
        let next = self.counters.get(&key).copied().unwrap_or(0) + 1;
        self.counters.insert(key.clone(), next);
        format!("{}-{:03}-{}", key, next, Self::hash4(&format!("{key}{next}")))
    }

    fn hash4(raw: &str) -> String {
        let h = Md5::digest(raw.as_bytes());
        format!("{:02x}{:02x}", h[0], h[1])
    }
}

// ---------------------------------------------------------------------
// Фильтры (пустая строка = все) + поиск + сортировка (по убыванию старта)
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskFilter {
    pub date_from: String,
    pub date_to: String,
    pub tags: Vec<String>,
    pub client: String,
    pub user: String,
    pub search: String,
}

/// Отобрать задачи по фильтру, новые сверху. Единая точка отбора для таблицы,
/// итогов, отчётов и статистики: так цифры везде совпадают.
pub fn filter_tasks<'a>(tasks: &'a [Task], f: &TaskFilter) -> Vec<&'a Task> {
    let from = parse_date(&f.date_from);
    let to = parse_date(&f.date_to);
    let q = f.search.trim().to_lowercase();
    let want_tags: Vec<String> = f
        .tags
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut out: Vec<&Task> = Vec::new();
    for t in tasks {
        if let Some(d) = from {
            if t.start_date().date() < d {
                continue;
            }
        }
        if let Some(d) = to {
            if t.start_date().date() > d {
                continue;
            }
        }
        if !want_tags.is_empty() {
            // Задача проходит, если выбраны некоторые/все её теги.
            let hit = t.tags.iter().any(|tag| want_tags.iter().any(|w| tag == w));
            if !hit {
                continue;
            }
        }
        if !f.client.trim().is_empty() && t.client != f.client.trim() {
            continue;
        }
        if !f.user.trim().is_empty() && t.user != f.user.trim() {
            continue;
        }
        if !q.is_empty() {
            let tags_text = t.tags.join(" ");
            let orders_text = t.orders.join(" ");
            let hay = [&t.task_id, &orders_text, &tags_text, &t.client, &t.user, &t.comment];
            if !hay.iter().any(|s| s.to_lowercase().contains(&q)) {
                continue;
            }
        }
        out.push(t);
    }
    out.sort_by_key(|t| std::cmp::Reverse(t.start_date()));
    out
}

/// Порядок строк основной таблицы по умолчанию: запущенные, на паузе, завершённые;
/// внутри каждой группы — по самому первому началу задачи (как в колонке «Начало»), новые сверху.
pub fn sort_for_table(tasks: &mut [&Task]) {
    tasks.sort_by_key(|t| (t.status.rank(), std::cmp::Reverse(t.start_date())));
}

pub fn apply_filter(tasks: &[Task], f: &TaskFilter, now: NaiveDateTime) -> Vec<AppTask> {
    filter_tasks(tasks, f)
        .into_iter()
        .map(|t| AppTask::from_task(t, now))
        .collect()
}

// ---------------------------------------------------------------------
// Итоги
// ---------------------------------------------------------------------

/// Два способа посчитать время набора задач.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TimeSums {
    /// «Сумма по задачам»: время каждой задачи складывается отдельно,
    /// одновременно идущие задачи считаются несколько раз.
    pub sum: f64,
    /// «Фактическое время»: объединение отрезков работы всех задач,
    /// пересечения по времени считаются один раз.
    pub actual: f64,
}

/// Секунды, в которые шла хотя бы одна из задач (отрезки объединяются).
pub fn union_seconds(tasks: &[&Task], now: NaiveDateTime) -> f64 {
    let mut spans: Vec<_> = tasks.iter().flat_map(|t| t.spans(now)).collect();
    spans.sort();
    let mut total = 0_i64;
    let mut current: Option<(NaiveDateTime, NaiveDateTime)> = None;
    for (start, stop) in spans {
        current = match current {
            Some((cs, ce)) if start <= ce => Some((cs, ce.max(stop))),
            Some((cs, ce)) => {
                total += (ce - cs).num_seconds();
                Some((start, stop))
            }
            None => Some((start, stop)),
        };
    }
    if let Some((cs, ce)) = current {
        total += (ce - cs).num_seconds();
    }
    total as f64
}

pub fn time_sums(tasks: &[&Task], now: NaiveDateTime) -> TimeSums {
    TimeSums {
        sum: tasks.iter().map(|t| t.total_seconds(now)).sum(),
        actual: union_seconds(tasks, now),
    }
}

/// Группировка по тегам: задача без тегов попадает в «(без тега)»,
/// задача с несколькими тегами — в каждую свою группу.
pub fn group_by_tag<'a>(tasks: &[&'a Task]) -> HashMap<String, Vec<&'a Task>> {
    let mut map: HashMap<String, Vec<&'a Task>> = HashMap::new();
    for &t in tasks {
        let tags: Vec<&str> = t.tags.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if t.tags.is_empty() {
            map.entry("(без тега)".to_string()).or_default().push(t);
        }
        for tag in tags {
            map.entry(tag.to_string()).or_default().push(t);
        }
    }
    map
}

/// Свод «тег → две суммы времени».
pub fn tag_sums(tasks: &[&Task], now: NaiveDateTime) -> HashMap<String, TimeSums> {
    group_by_tag(tasks)
        .into_iter()
        .map(|(name, group)| (name, time_sums(&group, now)))
        .collect()
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalLine {
    pub name: String,
    /// Сумма по задачам, секунды.
    pub seconds: f64,
    pub time_label: String,
    /// Фактическое время, секунды.
    pub actual_seconds: f64,
    pub actual_label: String,
}

impl TotalLine {
    pub fn new(name: String, sums: TimeSums) -> TotalLine {
        TotalLine {
            name,
            seconds: sums.sum,
            time_label: fmt_td(sums.sum),
            actual_seconds: sums.actual,
            actual_label: fmt_td(sums.actual),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub count: usize,
    /// Сумма по задачам, секунды.
    pub total_seconds: f64,
    pub time_label: String,
    /// Фактическое время, секунды.
    pub actual_seconds: f64,
    pub actual_label: String,
    pub by_tag: Vec<TotalLine>,
}

pub fn build_totals(tasks: &[&Task], now: NaiveDateTime) -> Totals {
    let sums = time_sums(tasks, now);
    let mut by_tag: Vec<TotalLine> = tag_sums(tasks, now)
        .into_iter()
        .map(|(name, s)| TotalLine::new(name, s))
        .collect();
    by_tag.sort_by(|a, b| b.seconds.partial_cmp(&a.seconds).unwrap_or(std::cmp::Ordering::Equal));
    Totals {
        count: tasks.len(),
        total_seconds: sums.sum,
        time_label: fmt_td(sums.sum),
        actual_seconds: sums.actual,
        actual_label: fmt_td(sums.actual),
        by_tag,
    }
}

// ---------------------------------------------------------------------
// Группировка и строки представления
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)] // строки таблицы собираются списком и сразу уходят в JSON
pub enum ViewRow {
    Group {
        label: String,
        count: usize,
        time_label: String,
    },
    Task(AppTask),
}

pub fn build_rows(tasks: &[AppTask], grouping: &str) -> Vec<ViewRow> {
    let mut rows: Vec<ViewRow> = Vec::new();
    match grouping {
        "day" => {
            let mut map: HashMap<String, (NaiveDate, Vec<AppTask>)> = HashMap::new();
            for t in tasks {
                let key = &t.start_label[..10];
                let entry = map
                    .entry(key.to_string())
                    .or_insert_with(|| (parse_date(key).unwrap_or_default(), Vec::new()));
                entry.1.push(t.clone());
            }
            let mut items: Vec<_> = map.into_values().collect();
            items.sort_by_key(|item| std::cmp::Reverse(item.0));
            for (date, group) in items {
                let total: f64 = group.iter().map(|t| t.seconds).sum();
                rows.push(ViewRow::Group {
                    label: date.format("%d.%m.%Y").to_string(),
                    count: group.len(),
                    time_label: fmt_td(total),
                });
                rows.extend(group.into_iter().map(ViewRow::Task));
            }
        }
        "client" => {
            let mut map: HashMap<String, (String, Vec<AppTask>)> = HashMap::new();
            for t in tasks {
                let key = if t.client.is_empty() {
                    "(не указан)".to_string()
                } else {
                    t.client.clone()
                };
                let entry = map.entry(key.clone()).or_insert((key, Vec::new()));
                entry.1.push(t.clone());
            }
            let mut items: Vec<_> = map.into_values().collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));
            for (client, group) in items {
                let total: f64 = group.iter().map(|t| t.seconds).sum();
                rows.push(ViewRow::Group {
                    label: client,
                    count: group.len(),
                    time_label: fmt_td(total),
                });
                rows.extend(group.into_iter().map(ViewRow::Task));
            }
        }
        _ => rows.extend(tasks.iter().cloned().map(ViewRow::Task)),
    }
    rows
}