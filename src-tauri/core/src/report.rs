//! Отчёты и статистика: фильтрация задач, предпросмотр,
//! запись TXT/MD/CSV(cp1251)/XLSX в папку reports/ (как Python-версия).

// Функции записи отчётов принимают набор независимых параметров формата и периода.
#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Local;
use serde::Serialize;

use crate::dt::{fmt_td, now_naive};
use crate::logic::{filter_tasks, group_by_tag, tag_sums, time_sums, TaskFilter, TimeSums, TotalLine};
use crate::models::{ColumnPref, Task, TaskStatus};

// ---------------------------------------------------------------------
// Колонки отчёта — как в таблице задач
// ---------------------------------------------------------------------

/// Колонки отчёта: ключ из настроек таблицы + заголовок.
/// Служебные колонки таблицы (actions) в отчёт не попадают.
const REPORT_COLUMNS: [(&str, &str); 12] = [
    ("time", "Время"),
    ("status", "Статус"),
    ("ourCar", "Наша машина"),
    ("mode", "Режим"),
    ("order", "Заявка"),
    ("client", "Клиент"),
    ("tags", "Тег"),
    ("start", "Начало"),
    ("end", "Завершение"),
    ("user", "Пользователь"),
    ("comment", "Комментарий"),
    ("taskId", "ID"),
];

/// Эффективный список колонок отчёта по настройкам таблицы: только видимые,
/// в сохранённом порядке; колонки, не упомянутые в настройках, добавляются
/// в конец. Скрытые колонки не выгружаются.
fn report_column_keys(prefs: &[ColumnPref]) -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for p in prefs {
        if !p.visible || p.key == "actions" {
            continue;
        }
        if let Some((k, _)) = REPORT_COLUMNS.iter().find(|(k, _)| *k == p.key) {
            if !out.contains(k) {
                out.push(k);
            }
        }
    }
    for (k, _) in REPORT_COLUMNS {
        let mentioned = prefs.iter().any(|p| p.key == k);
        if !mentioned && !out.contains(&k) {
            out.push(k);
        }
    }
    out
}

fn report_column_title(key: &str) -> &'static str {
    REPORT_COLUMNS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, t)| *t)
        .unwrap_or("")
}

/// Человекочитаемая метка режима задачи (как в таблице).
fn mode_label(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Running => "В работе",
        TaskStatus::Paused => "На паузе",
        TaskStatus::Completed => "Завершена",
    }
}

/// Значение ячейки отчёта для колонки таблицы.
fn report_cell(t: &Task, key: &str, now: chrono::NaiveDateTime) -> String {
    match key {
        "time" => fmt_td(t.total_seconds(now)),
        "status" => t.custom_status.clone(),
        "ourCar" => {
            if t.our_car {
                "Да".to_string()
            } else {
                String::new()
            }
        }
        "mode" => mode_label(&t.status).to_string(),
        "order" => t.orders.join(", "),
        "client" => t.client.clone(),
        "tags" => t.tags.join(", "),
        "start" => t.start_str(),
        "end" => t.end_str(),
        "user" => t.user.clone(),
        "comment" => t.comment.clone(),
        "taskId" => t.task_id.clone(),
        _ => String::new(),
    }
}

/// Распарсить "#rrggbb" или "#rrggbbaa" в u32 (0xRRGGBBAA). Err — для RGB белый.
fn accent_u32(s: &str) -> u32 {
    let t = s.trim().strip_prefix('#').unwrap_or(s.trim());
    if t.len() >= 6 && t[..6].chars().all(|c| c.is_ascii_hexdigit()) {
        u32::from_str_radix(&t[..6], 16).unwrap_or(0xFFFFFFFF)
    } else {
        0xFFFFFFFF
    }
}

// ---------------------------------------------------------------------
// Предпросмотр отчёта
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportRow {
    pub task_id: String,
    pub start: String,
    pub end: String,
    pub user: String,
    pub order: String,
    pub client: String,
    pub tags: Vec<String>,
    pub elapsed_label: String,
    pub comment: String,
    pub ranges: String,
    pub custom_status: String,
    pub our_car: bool,
    pub mode: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPreview {
    pub rows: Vec<ReportRow>,
    pub count: usize,
    /// Сумма по задачам.
    pub total_label: String,
    /// Фактическое время (пересечения считаются один раз).
    pub actual_label: String,
    pub by_tag: Vec<TotalLine>,
}

// ---------------------------------------------------------------------
// Статистика
// ---------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsLine {
    pub name: String,
    pub count: usize,
    /// Сумма по задачам.
    pub total_label: String,
    /// Фактическое время (пересечения считаются один раз).
    pub actual_label: String,
    pub avg_label: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub count: usize,
    /// Сумма по задачам.
    pub total_label: String,
    /// Фактическое время (пересечения считаются один раз).
    pub actual_label: String,
    pub avg_label: String,
    pub by_user: Vec<StatsLine>,
    pub by_client: Vec<StatsLine>,
    pub by_tag: Vec<StatsLine>,
    pub by_month: Vec<StatsLine>,
    pub by_client_user: Vec<StatsLine>,
}

// ---------------------------------------------------------------------
// Общие вспомогательные функции
// ---------------------------------------------------------------------

/// Отфильтровать задачи по фильтру (даты — по дате начала), сортировка
/// по убыванию старта (как Python `Store.filtered`). Поиск не используется,
/// но поддерживается фильтром.
pub fn filtered_tasks<'a>(tasks: &'a [Task], f: &TaskFilter) -> Vec<&'a Task> {
    filter_tasks(tasks, f)
}

fn stats_line(name: String, group: &[&Task], now: chrono::NaiveDateTime) -> (f64, StatsLine) {
    let sums = time_sums(group, now);
    let count = group.len();
    let avg = if count > 0 { sums.sum / count as f64 } else { 0.0 };
    let line = StatsLine {
        name,
        count,
        total_label: fmt_td(sums.sum),
        actual_label: fmt_td(sums.actual),
        avg_label: fmt_td(avg),
    };
    (sums.sum, line)
}

/// Строки «Имя | Задач | Сумма по задачам | Фактическое время | Среднее»,
/// сортировка по сумме по задачам. Фактическое время считается внутри каждой группы.
fn lines_from_groups(map: HashMap<String, Vec<&Task>>, now: chrono::NaiveDateTime) -> Vec<StatsLine> {
    let mut lines: Vec<(f64, StatsLine)> = map
        .into_iter()
        .map(|(name, group)| stats_line(name, &group, now))
        .collect();
    lines.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    lines.into_iter().map(|(_, line)| line).collect()
}

fn group_stats<'a>(tasks: &[&'a Task], key: impl Fn(&'a Task) -> String, now: chrono::NaiveDateTime) -> Vec<StatsLine> {
    let mut map: HashMap<String, Vec<&'a Task>> = HashMap::new();
    for &t in tasks {
        map.entry(key(t)).or_default().push(t);
    }
    lines_from_groups(map, now)
}

/// Разбивка «по тегам»: многозначная задача входит во все свои теги.
fn tag_stats(list: &[&Task], now: chrono::NaiveDateTime) -> Vec<StatsLine> {
    lines_from_groups(group_by_tag(list), now)
}

// ---------------------------------------------------------------------
// Report preview
// ---------------------------------------------------------------------

pub fn report_preview(tasks: &[Task], f: &TaskFilter) -> ReportPreview {
    let now = now_naive();
    let list = filtered_tasks(tasks, f);
    let sums = time_sums(&list, now);
    let rows: Vec<ReportRow> = list
        .iter()
        .map(|t| ReportRow {
            task_id: t.task_id.clone(),
            start: t.start_str(),
            end: t.end_str(),
            user: t.user.clone(),
            order: t.orders.join(", "),
            client: t.client.clone(),
            tags: t.tags.clone(),
            elapsed_label: fmt_td(t.total_seconds(now)),
            comment: t.comment.clone(),
            ranges: t.ranges_str(),
            custom_status: t.custom_status.clone(),
            our_car: t.our_car,
            mode: mode_label(&t.status).to_string(),
        })
        .collect();

    let mut by_tag: Vec<TotalLine> = tag_sums(&list, now)
        .into_iter()
        .map(|(name, s)| TotalLine::new(name, s))
        .collect();
    by_tag.sort_by(|a, b| b.seconds.partial_cmp(&a.seconds).unwrap_or(std::cmp::Ordering::Equal));

    ReportPreview {
        rows,
        count: list.len(),
        total_label: fmt_td(sums.sum),
        actual_label: fmt_td(sums.actual),
        by_tag,
    }
}

// ---------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------

pub fn statistics(tasks: &[Task], f: &TaskFilter) -> Stats {
    let now = now_naive();
    let list = filtered_tasks(tasks, f);
    let sums = time_sums(&list, now);
    let count = list.len();

    let by_user = group_stats(&list, |t| {
        if t.user.is_empty() {
            "(не указан)".to_string()
        } else {
            t.user.clone()
        }
    }, now);
    let by_client = group_stats(&list, |t| {
        if t.client.is_empty() {
            "(не указан)".to_string()
        } else {
            t.client.clone()
        }
    }, now);
    let by_tag = tag_stats(&list, now);
    let by_month = group_stats(&list, |t| t.start_date().format("%Y-%m").to_string(), now);
    let by_client_user = group_stats(&list, |t| {
        let or_none = |s: &str| if s.is_empty() { "(не указан)".to_string() } else { s.to_string() };
        format!("{} / {}", or_none(&t.client), or_none(&t.user))
    }, now);

    Stats {
        count,
        total_label: fmt_td(sums.sum),
        actual_label: fmt_td(sums.actual),
        avg_label: fmt_td(if count > 0 { sums.sum / count as f64 } else { 0.0 }),
        by_user,
        by_client,
        by_tag,
        by_month,
        by_client_user,
    }
}

// ---------------------------------------------------------------------
// Запись файлов отчётов/статистики
// ---------------------------------------------------------------------

fn reports_dir(base: &Path) -> PathBuf {
    let d = base.join("reports");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn ts_stamp() -> String {
    Local::now().format("%Y%m%d_%H%M%S").to_string()
}

/// Вернуть путь к сохранённому файлу или Err с текстом ошибки.
/// Сохраняет в reports/ базового каталога.
pub fn write_report(
    base: &Path,
    format: &str,
    tasks: &[Task],
    filter: &TaskFilter,
    date_from: &str,
    date_to: &str,
    accent_color: &str,
    columns: &[ColumnPref],
) -> Result<PathBuf, String> {
    let now = now_naive();
    let list = filtered_tasks(tasks, filter);
    if list.is_empty() {
        return Err("Нет записей за выбранный период".to_string());
    }
    let sums = time_sums(&list, now);
    let tag_map = tag_sums(&list, now);

    let dir = reports_dir(base);
    let ts = ts_stamp();
    let path = dir.join(format!("report_{ts}.{format}"));
    write_report_body(&path, format, &list, sums, &tag_map, date_from, date_to, accent_color, columns)?;
    Ok(path)
}

/// Сохранить отчёт по явно заданному пути (диалог выбора файла).
pub fn write_report_to(
    path: &Path,
    format: &str,
    tasks: &[Task],
    filter: &TaskFilter,
    date_from: &str,
    date_to: &str,
    accent_color: &str,
    columns: &[ColumnPref],
) -> Result<PathBuf, String> {
    let now = now_naive();
    let list = filtered_tasks(tasks, filter);
    if list.is_empty() {
        return Err("Нет записей за выбранный период".to_string());
    }
    let sums = time_sums(&list, now);
    let tag_map = tag_sums(&list, now);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    write_report_body(path, format, &list, sums, &tag_map, date_from, date_to, accent_color, columns)?;
    Ok(path.to_path_buf())
}

fn write_report_body(
    path: &Path,
    format: &str,
    list: &[&Task],
    sums: TimeSums,
    tag_map: &HashMap<String, TimeSums>,
    date_from: &str,
    date_to: &str,
    accent_color: &str,
    columns: &[ColumnPref],
) -> Result<(), String> {
    match format {
        "txt" => {
            let body = report_txt(list, sums, tag_map, date_from, date_to, now_naive());
            std::fs::write(path, body).map_err(|e| e.to_string())?;
        }
        "md" => {
            let body = report_md(list, sums, tag_map, date_from, date_to);
            std::fs::write(path, body).map_err(|e| e.to_string())?;
        }
        "csv" => {
            let keys = report_column_keys(columns);
            let mut body = String::new();
            body.push_str(&row_csv(
                &keys.iter().map(|k| report_column_title(k).to_string()).collect::<Vec<_>>(),
            ));
            let now = now_naive();
            for t in list {
                body.push_str(&row_csv(
                    &keys.iter().map(|k| report_cell(t, k, now)).collect::<Vec<_>>(),
                ));
            }
            write_cp1251(path, &body)?;
        }
        "xlsx" => {
            write_report_xlsx(path, list, sums, tag_map, date_from, date_to, accent_color, columns)?;
        }
        _ => return Err(format!("Неизвестный формат: {format}")),
    }
    Ok(())
}

/// Путь к файлу статистики или Err. Сохраняет в reports/ базового каталога.
pub fn write_stats(
    base: &Path,
    format: &str,
    tasks: &[Task],
    filter: &TaskFilter,
    date_from: &str,
    date_to: &str,
    accent_color: &str,
) -> Result<PathBuf, String> {
    let st = statistics(tasks, filter);
    if st.count == 0 {
        return Err("Нет записей за выбранный период".to_string());
    }
    let dir = reports_dir(base);
    let ts = ts_stamp();
    let path = dir.join(format!("stats_{ts}.{format}"));
    write_stats_body(&path, format, &st, date_from, date_to, accent_color)?;
    Ok(path)
}

/// Сохранить статистику по явно заданному пути (диалог выбора файла).
pub fn write_stats_to(
    path: &Path,
    format: &str,
    tasks: &[Task],
    filter: &TaskFilter,
    date_from: &str,
    date_to: &str,
    accent_color: &str,
) -> Result<PathBuf, String> {
    let st = statistics(tasks, filter);
    if st.count == 0 {
        return Err("Нет записей за выбранный период".to_string());
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    write_stats_body(path, format, &st, date_from, date_to, accent_color)?;
    Ok(path.to_path_buf())
}

fn write_stats_body(
    path: &Path,
    format: &str,
    st: &Stats,
    date_from: &str,
    date_to: &str,
    accent_color: &str,
) -> Result<(), String> {
    match format {
        "txt" => {
            let body = stats_txt(st, date_from, date_to);
            std::fs::write(path, body).map_err(|e| e.to_string())?;
        }
        "md" => {
            let body = stats_md(st, date_from, date_to);
            std::fs::write(path, body).map_err(|e| e.to_string())?;
        }
        "csv" => {
            let mut body = String::new();
            body.push_str("\"СТАТИСТИКА\",\"Период: ");
            body.push_str(&fmt_period(date_from, date_to));
            body.push_str("\"\r\n");
            body.push_str(&row_csv(&[
                "Записей".to_string(),
                st.count.to_string(),
                "Сумма по задачам".to_string(),
                st.total_label.clone(),
                "Фактическое время".to_string(),
                st.actual_label.clone(),
            ]));
            body.push_str("\r\n");
            for (title, items) in stats_sections(st) {
                if items.is_empty() {
                    continue;
                }
                body.push_str(&row_csv(&[title.to_string()]));
                body.push_str(&row_csv(&[
                    "Имя".to_string(),
                    "Задач".to_string(),
                    "Сумма по задачам".to_string(),
                    "Фактическое время".to_string(),
                    "Среднее".to_string(),
                ]));
                for item in items {
                    body.push_str(&row_csv(&[
                        item.name.clone(),
                        item.count.to_string(),
                        item.total_label.clone(),
                        item.actual_label.clone(),
                        item.avg_label.clone(),
                    ]));
                }
                body.push_str("\r\n");
            }
            write_cp1251(path, &body)?;
        }
        "xlsx" => {
            write_stats_xlsx(path, st, date_from, date_to, accent_color)?;
        }
        _ => return Err(format!("Неизвестный формат: {format}")),
    }
    Ok(())
}

fn fmt_period(date_from: &str, date_to: &str) -> String {
    format!("{date_from} — {date_to}")
}

fn report_txt(list: &[&Task], sums: TimeSums, tag_map: &HashMap<String, TimeSums>, date_from: &str, date_to: &str, now: chrono::NaiveDateTime) -> String {
    let mut lines = Vec::new();
    lines.push("ОТЧЁТ ПО ВРЕМЕНИ".to_string());
    lines.push(format!("Период: {}", fmt_period(date_from, date_to)));
    lines.push("=".repeat(70));
    lines.push(String::new());
    lines.push(format!("Записей: {}", list.len()));
    lines.push(format!("Сумма по задачам: {}", fmt_td(sums.sum)));
    lines.push(format!("Фактическое время: {}", fmt_td(sums.actual)));
    lines.push(String::new());
    lines.push("ПО ТЕГАМ (сумма по задачам / фактическое время):".to_string());
    lines.push("-".repeat(40));
    let mut tags: Vec<(&String, &TimeSums)> = tag_map.iter().collect();
    tags.sort_by(|a, b| a.0.cmp(b.0));
    for (name, s) in tags {
        lines.push(format!("  {:<30} {}  /  {}", name, fmt_td(s.sum), fmt_td(s.actual)));
    }
    lines.push(String::new());
    lines.push("ДЕТАЛИЗАЦИЯ:".to_string());
    lines.push("-".repeat(70));
    for t in list {
        lines.push(format!(
            "  {:<16} {:<16} {:<15} {:<15} {:<10} {}  #{}  | Диапазоны: {}",
            t.start_str(),
            t.end_str(),
            t.user,
            t.client,
            t.tags.join(", "),
            fmt_td(t.total_seconds(now)),
            t.orders.join(", "),
            t.ranges_str(),
        ));
        if !t.comment.is_empty() {
            lines.push(format!("    Комментарий: {}", t.comment));
        }
    }
    lines.join("\n")
}

fn report_md(list: &[&Task], sums: TimeSums, tag_map: &HashMap<String, TimeSums>, date_from: &str, date_to: &str) -> String {
    let mut lines = Vec::new();
    lines.push("# Отчёт по времени\n".to_string());
    lines.push(format!("**Период:** {}\n", fmt_period(date_from, date_to)));
    lines.push(format!("**Записей:** {}  ", list.len()));
    lines.push(format!("**Сумма по задачам:** {}  ", fmt_td(sums.sum)));
    lines.push(format!("**Фактическое время:** {}\n", fmt_td(sums.actual)));
    lines.push("## По тегам\n".to_string());
    lines.push("| Тег | Сумма по задачам | Фактическое время |".to_string());
    lines.push("|-----|------------------|-------------------|".to_string());
    let mut tags: Vec<(&String, &TimeSums)> = tag_map.iter().collect();
    tags.sort_by(|a, b| a.0.cmp(b.0));
    for (name, s) in tags {
        lines.push(format!("| {} | {} | {} |", name, fmt_td(s.sum), fmt_td(s.actual)));
    }
    lines.push(String::new());
    lines.push("## Детализация\n".to_string());
    lines.push("| Дата начала | Дата окончания | Пользователь | Клиент | Тег | Время | Заявка | Комментарий | Диапазоны |".to_string());
    lines.push("|-------------|----------------|--------------|--------|-----|-------|--------|-------------|-----------|".to_string());
    for t in list {
        let comment = t.comment.replace('\n', " ");
        let comment = if comment.chars().count() > 30 {
            comment.chars().take(30).collect::<String>()
        } else {
            comment
        };
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            t.start_str(),
            t.end_str(),
            t.user,
            t.client,
            t.tags.join(", "),
            fmt_td(t.total_seconds(now_naive())),
            t.orders.join(", "),
            comment,
            t.ranges_str(),
        ));
    }
    lines.join("\n")
}

fn row_csv(fields: &[String]) -> String {
    let mut out = Vec::new();
    for f in fields {
        out.push(format!("\"{}\"", f.replace('"', "\"\"")));
    }
    out.join(",") + "\r\n"
}

fn write_cp1251(path: &Path, body: &str) -> Result<(), String> {
    let (bytes, _, _) = encoding_rs::WINDOWS_1251.encode(body);
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

fn header_format(accent: u32) -> rust_xlsxwriter::Format {
    rust_xlsxwriter::Format::new()
        .set_bold()
        .set_font_color(rust_xlsxwriter::Color::White)
        .set_background_color(rust_xlsxwriter::Color::RGB(accent))
        .set_align(rust_xlsxwriter::FormatAlign::Center)
        .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter)
        .set_border(rust_xlsxwriter::FormatBorder::Thin)
}

fn write_report_xlsx(
    path: &Path,
    list: &[&Task],
    sums: TimeSums,
    tag_map: &HashMap<String, TimeSums>,
    date_from: &str,
    date_to: &str,
    accent_color: &str,
    columns: &[ColumnPref],
) -> Result<(), String> {
    use rust_xlsxwriter::Workbook;

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Отчёт").map_err(|e| e.to_string())?;

    let header = header_format(accent_u32(accent_color));
    let mut row = 0u32;
    ws.write_string_with_format(row, 0, "ОТЧЁТ ПО ВРЕМЕНИ", &header).map_err(err)?;
    row += 1;
    ws.write_string(row, 0, fmt_period(date_from, date_to)).map_err(err)?;
    row += 2;
    ws.write_string(row, 0, format!("Записей: {}", list.len())).map_err(err)?;
    ws.write_string(row, 1, format!("Сумма по задачам: {}", fmt_td(sums.sum))).map_err(err)?;
    ws.write_string(row, 2, format!("Фактическое время: {}", fmt_td(sums.actual))).map_err(err)?;
    row += 2;
    ws.write_string(row, 0, "По тегам:").map_err(err)?;
    ws.write_string(row, 1, "Сумма по задачам").map_err(err)?;
    ws.write_string(row, 2, "Фактическое время").map_err(err)?;
    row += 1;
    let mut tags: Vec<(&String, &TimeSums)> = tag_map.iter().collect();
    tags.sort_by(|a, b| a.0.cmp(b.0));
    for (name, s) in tags {
        ws.write_string(row, 0, name).map_err(err)?;
        ws.write_string(row, 1, fmt_td(s.sum)).map_err(err)?;
        ws.write_string(row, 2, fmt_td(s.actual)).map_err(err)?;
        row += 1;
    }
    row += 1;

    let keys = report_column_keys(columns);
    for (col, k) in keys.iter().enumerate() {
        ws.write_string_with_format(row, col as u16, report_column_title(k), &header).map_err(err)?;
    }
    row += 1;
    let now = now_naive();
    for t in list {
        for (col, k) in keys.iter().enumerate() {
            ws.write_string(row, col as u16, report_cell(t, k, now)).map_err(err)?;
        }
        row += 1;
    }

    for col in 0..keys.len() {
        ws.set_column_width(col as u16, 18.0).map_err(err)?;
    }
    wb.save(path).map_err(err)
}

fn stats_sections(st: &Stats) -> [(&'static str, &[StatsLine]); 5] {
    [
        ("По пользователям", &st.by_user),
        ("По клиентам", &st.by_client),
        ("По тегам", &st.by_tag),
        ("По месяцам", &st.by_month),
        ("Клиент / пользователь", &st.by_client_user),
    ]
}

fn stats_txt(st: &Stats, date_from: &str, date_to: &str) -> String {
    let mut lines = Vec::new();
    lines.push("СТАТИСТИКА".to_string());
    lines.push(format!("Период: {}", fmt_period(date_from, date_to)));
    lines.push("=".repeat(70));
    lines.push(String::new());
    lines.push(format!("Записей: {}", st.count));
    lines.push(format!("Сумма по задачам: {}", st.total_label));
    lines.push(format!("Фактическое время: {}", st.actual_label));
    lines.push(format!("Среднее время задачи: {}", st.avg_label));
    lines.push(String::new());
    for (title, items) in stats_sections(st) {
        if items.is_empty() {
            continue;
        }
        lines.push(format!("{title}:"));
        lines.push("  (сумма по задачам / фактическое время)".to_string());
        lines.push("-".repeat(70));
        for item in items {
            lines.push(format!(
                "  {:<25} {:>6} задач {:>10} / {:<10} (ср. {})",
                item.name, item.count, item.total_label, item.actual_label, item.avg_label
            ));
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn stats_md(st: &Stats, date_from: &str, date_to: &str) -> String {
    let mut lines = Vec::new();
    lines.push("# Статистика\n".to_string());
    lines.push(format!("**Период:** {}\n", fmt_period(date_from, date_to)));
    lines.push(format!("**Записей:** {}  ", st.count));
    lines.push(format!("**Сумма по задачам:** {}  ", st.total_label));
    lines.push(format!("**Фактическое время:** {}  ", st.actual_label));
    lines.push(format!("**Среднее:** {}\n", st.avg_label));
    for (title, items) in stats_sections(st) {
        if items.is_empty() {
            continue;
        }
        lines.push(format!("## {title}\n"));
        lines.push("| Имя | Задач | Сумма по задачам | Фактическое время | Среднее |".to_string());
        lines.push("|-----|-------|------------------|-------------------|---------|".to_string());
        for item in items {
            lines.push(format!(
                "| {} | {} | {} | {} | {} |",
                item.name, item.count, item.total_label, item.actual_label, item.avg_label
            ));
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn write_stats_xlsx(path: &Path, st: &Stats, date_from: &str, date_to: &str, accent_color: &str) -> Result<(), String> {
    use rust_xlsxwriter::Workbook;

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Статистика").map_err(err)?;

    let header = header_format(accent_u32(accent_color));
    let mut row = 0u32;
    ws.write_string_with_format(row, 0, "СТАТИСТИКА", &header).map_err(err)?;
    row += 1;
    ws.write_string(row, 0, fmt_period(date_from, date_to)).map_err(err)?;
    row += 2;
    ws.write_string(row, 0, format!("Записей: {}", st.count)).map_err(err)?;
    ws.write_string(row, 1, format!("Сумма по задачам: {}", st.total_label)).map_err(err)?;
    ws.write_string(row, 2, format!("Фактическое время: {}", st.actual_label)).map_err(err)?;
    ws.write_string(row, 3, format!("Среднее: {}", st.avg_label)).map_err(err)?;
    row += 2;

    for (title, items) in stats_sections(st) {
        if items.is_empty() {
            continue;
        }
        ws.write_string(row, 0, title).map_err(err)?;
        row += 1;
        for (col, h) in ["Имя", "Задач", "Сумма по задачам", "Фактическое время", "Среднее"].iter().enumerate() {
            ws.write_string_with_format(row, col as u16, *h, &header).map_err(err)?;
        }
        row += 1;
        for item in items {
            ws.write_string(row, 0, &item.name).map_err(err)?;
            ws.write_number(row, 1, item.count as f64).map_err(err)?;
            ws.write_string(row, 2, &item.total_label).map_err(err)?;
            ws.write_string(row, 3, &item.actual_label).map_err(err)?;
            ws.write_string(row, 4, &item.avg_label).map_err(err)?;
            row += 1;
        }
        row += 1;
    }
    for col in 0..5u16 {
        ws.set_column_width(col, 18.0).map_err(err)?;
    }
    wb.save(path).map_err(err)
}

fn err(e: rust_xlsxwriter::XlsxError) -> String {
    e.to_string()
}
