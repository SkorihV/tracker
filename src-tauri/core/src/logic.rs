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
    pub tag: String,
    pub client: String,
    pub user: String,
    pub search: String,
}

pub fn apply_filter(tasks: &[Task], f: &TaskFilter, now: NaiveDateTime) -> Vec<AppTask> {
    let from = parse_date(&f.date_from);
    let to = parse_date(&f.date_to);
    let q = f.search.trim().to_lowercase();

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
        if !f.tag.trim().is_empty() && t.tag != f.tag.trim() {
            continue;
        }
        if !f.client.trim().is_empty() && t.client != f.client.trim() {
            continue;
        }
        if !f.user.trim().is_empty() && t.user != f.user.trim() {
            continue;
        }
        if !q.is_empty() {
            let hay = [&t.task_id, &t.order, &t.tag, &t.client, &t.user, &t.comment];
            if !hay.iter().any(|s| s.to_lowercase().contains(&q)) {
                continue;
            }
        }
        out.push(t);
    }
    out.sort_by(|a, b| b.start_date().cmp(&a.start_date()));
    out.into_iter().map(|t| AppTask::from_task(t, now)).collect()
}

// ---------------------------------------------------------------------
// Итоги
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalLine {
    pub name: String,
    pub seconds: f64,
    pub time_label: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub count: usize,
    pub total_seconds: f64,
    pub time_label: String,
    pub by_tag: Vec<TotalLine>,
}

pub fn build_totals(tasks: &[AppTask]) -> Totals {
    let total: f64 = tasks.iter().map(|t| t.seconds).sum();
    let mut map: HashMap<String, f64> = HashMap::new();
    for t in tasks {
        let k = if t.tag.is_empty() {
            "(без тега)"
        } else {
            t.tag.as_str()
        };
        *map.entry(k.to_string()).or_insert(0.0) += t.seconds;
    }
    let mut by_tag: Vec<TotalLine> = map
        .into_iter()
        .map(|(name, seconds)| TotalLine {
            name,
            seconds,
            time_label: fmt_td(seconds),
        })
        .collect();
    by_tag.sort_by(|a, b| b.seconds.partial_cmp(&a.seconds).unwrap_or(std::cmp::Ordering::Equal));
    Totals {
        count: tasks.len(),
        total_seconds: total,
        time_label: fmt_td(total),
        by_tag,
    }
}

// ---------------------------------------------------------------------
// Группировка и строки представления
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
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
            items.sort_by(|a, b| b.0.cmp(&a.0));
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