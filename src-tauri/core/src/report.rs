//! Отчёты и статистика: фильтрация задач, предпросмотр,
//! запись TXT/MD/CSV(cp1251)/XLSX в папку reports/ (как Python-версия).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Local;
use serde::Serialize;

use crate::dt::{fmt_td, now_naive, parse_date};
use crate::logic::{TaskFilter, TotalLine};
use crate::models::Task;

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
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPreview {
    pub rows: Vec<ReportRow>,
    pub count: usize,
    pub total_label: String,
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
    pub total_label: String,
    pub avg_label: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub count: usize,
    pub total_label: String,
    pub avg_label: String,
    pub by_user: Vec<StatsLine>,
    pub by_client: Vec<StatsLine>,
    pub by_tag: Vec<StatsLine>,
    pub by_month: Vec<StatsLine>,
}

// ---------------------------------------------------------------------
// Общие вспомогательные функции
// ---------------------------------------------------------------------

/// Отфильтровать задачи по фильтру (даты — по дате начала), сортировка
/// по убыванию старта (как Python `Store.filtered`). Поиск не используется,
/// но поддерживается фильтром.
pub fn filtered_tasks<'a>(tasks: &'a [Task], f: &TaskFilter) -> Vec<&'a Task> {
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
            let hay = [&t.task_id, &t.order, &tags_text, &t.client, &t.user, &t.comment];
            if !hay.iter().any(|s| s.to_lowercase().contains(&q)) {
                continue;
            }
        }
        out.push(t);
    }
    out.sort_by(|a, b| b.start_date().cmp(&a.start_date()));
    out
}

fn total_seconds<'a, I>(items: I, now: chrono::NaiveDateTime) -> f64
where
    I: IntoIterator<Item = &'a Task>,
{
    items.into_iter().map(|t| t.total_seconds(now)).sum()
}

/// Свод «тег → секунды»: задача без тегов попадает в «(без тега)»,
/// многозначная — в каждую строку своего тега.
fn tag_seconds<'a, I>(items: I, now: chrono::NaiveDateTime) -> HashMap<String, f64>
where
    I: IntoIterator<Item = &'a Task>,
{
    let mut map: HashMap<String, f64> = HashMap::new();
    for t in items {
        if t.tags.is_empty() {
            *map.entry("(без тега)".to_string()).or_insert(0.0) += t.total_seconds(now);
        } else {
            for tag in &t.tags {
                if tag.trim().is_empty() {
                    continue;
                }
                *map.entry(tag.trim().to_string()).or_insert(0.0) += t.total_seconds(now);
            }
        }
    }
    map
}

/// Разбивка «Имя | Задач | Общее время | Среднее», сортировка по времени.
fn group_stats<'a>(tasks: &[&'a Task], key: impl Fn(&'a Task) -> String, now: chrono::NaiveDateTime) -> Vec<StatsLine> {
    let mut map: HashMap<String, Vec<&Task>> = HashMap::new();
    for t in tasks {
        map.entry(key(t)).or_default().push(t);
    }
    let mut lines: Vec<StatsLine> = map
        .into_iter()
        .map(|(name, group)| {
            let secs = total_seconds(group.iter().copied(), now);
            let count = group.len();
            StatsLine {
                name,
                count,
                total_label: fmt_td(secs),
                avg_label: fmt_td(if count > 0 { secs / count as f64 } else { 0.0 }),
            }
        })
        .collect();
    lines.sort_by(|a, b| {
        let ok = fmt_to_secs(&b.total_label)
            .partial_cmp(&fmt_to_secs(&a.total_label))
            .unwrap_or(std::cmp::Ordering::Equal);
        ok
    });
    lines
}

fn fmt_to_secs(label: &str) -> f64 {
    let parts: Vec<&str> = label.split(':').collect();
    if parts.len() != 3 {
        return 0.0;
    }
    let h: f64 = parts[0].trim().parse().unwrap_or(0.0);
    let m: f64 = parts[1].trim().parse().unwrap_or(0.0);
    let s: f64 = parts[2].trim().parse().unwrap_or(0.0);
    h * 3600.0 + m * 60.0 + s
}

/// Разбивка «по тегам» для статистики: многозначная задача входит во все
/// свои теги. «(без тега)» — задач без тегов.
fn tag_stats<'a>(list: &[&'a Task], now: chrono::NaiveDateTime) -> Vec<StatsLine> {
    let mut map: HashMap<String, Vec<&'a Task>> = HashMap::new();
    for t in list {
        if t.tags.is_empty() {
            map.entry("(без тега)".to_string()).or_default().push(t);
        } else {
            for tag in &t.tags {
                if tag.trim().is_empty() {
                    continue;
                }
                map.entry(tag.trim().to_string()).or_default().push(t);
            }
        }
    }
    let mut lines: Vec<StatsLine> = map
        .into_iter()
        .map(|(name, group)| {
            let secs = total_seconds(group.iter().copied(), now);
            let count = group.len();
            StatsLine {
                name,
                count,
                total_label: fmt_td(secs),
                avg_label: fmt_td(if count > 0 { secs / count as f64 } else { 0.0 }),
            }
        })
        .collect();
    lines.sort_by(|a, b| {
        fmt_to_secs(&b.total_label)
            .partial_cmp(&fmt_to_secs(&a.total_label))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    lines
}

// ---------------------------------------------------------------------
// Report preview
// ---------------------------------------------------------------------

pub fn report_preview(tasks: &[Task], f: &TaskFilter) -> ReportPreview {
    let now = now_naive();
    let list = filtered_tasks(tasks, f);
    let secs = total_seconds(list.iter().copied(), now);
    let rows: Vec<ReportRow> = list
        .iter()
        .map(|t| ReportRow {
            task_id: t.task_id.clone(),
            start: t.start_str(),
            end: t.end_str(),
            user: t.user.clone(),
            order: t.order.clone(),
            client: t.client.clone(),
            tags: t.tags.clone(),
            elapsed_label: fmt_td(t.total_seconds(now)),
            comment: t.comment.clone(),
            ranges: t.ranges_str(),
        })
        .collect();

    let mut by_tag: Vec<TotalLine> = tag_seconds(list.iter().copied(), now)
        .into_iter()
        .map(|(name, seconds)| TotalLine {
            name,
            seconds,
            time_label: fmt_td(seconds),
        })
        .collect();
    by_tag.sort_by(|a, b| b.seconds.partial_cmp(&a.seconds).unwrap_or(std::cmp::Ordering::Equal));

    ReportPreview {
        rows,
        count: list.len(),
        total_label: fmt_td(secs),
        by_tag,
    }
}

// ---------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------

pub fn statistics(tasks: &[Task], f: &TaskFilter) -> Stats {
    let now = now_naive();
    let list = filtered_tasks(tasks, f);
    let secs = total_seconds(list.iter().copied(), now);
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

    Stats {
        count,
        total_label: fmt_td(secs),
        avg_label: fmt_td(if count > 0 { secs / count as f64 } else { 0.0 }),
        by_user,
        by_client,
        by_tag,
        by_month,
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
pub fn write_report(
    base: &Path,
    format: &str,
    tasks: &[Task],
    filter: &TaskFilter,
    date_from: &str,
    date_to: &str,
) -> Result<PathBuf, String> {
    let now = now_naive();
    let list = filtered_tasks(tasks, filter);
    if list.is_empty() {
        return Err("Нет записей за выбранный период".to_string());
    }
    let secs = total_seconds(list.iter().copied(), now);
    let tag_map = tag_seconds(list.iter().copied(), now);

    let dir = reports_dir(base);
    let ts = ts_stamp();
    let path = dir.join(format!("report_{ts}.{format}"));

    match format {
        "txt" => {
            let body = report_txt(&list, secs, &tag_map, date_from, date_to, now);
            std::fs::write(&path, body).map_err(|e| e.to_string())?;
        }
        "md" => {
            let body = report_md(&list, secs, &tag_map, date_from, date_to);
            std::fs::write(&path, body).map_err(|e| e.to_string())?;
        }
        "csv" => {
            let mut body = String::new();
            body.push_str("\"Дата начала\",\"Дата окончания\",\"Пользователь\",\"Номер заявки\",\"Клиент\",\"Тег\",\"Время (чч:мм:сс)\",\"Комментарий\",\"Диапазоны\"\r\n");
            for t in &list {
                body.push_str(&row_csv(&[
                    t.start_str(),
                    t.end_str(),
                    t.user.clone(),
                    t.order.clone(),
                    t.client.clone(),
                    t.tags.join(", "),
                    fmt_td(t.total_seconds(now)),
                    t.comment.clone(),
                    t.ranges_str(),
                ]));
            }
            write_cp1251(&path, &body)?;
        }
        "xlsx" => {
            write_report_xlsx(&path, &list, secs, &tag_map, date_from, date_to)?;
        }
        _ => return Err(format!("Неизвестный формат: {format}")),
    }
    Ok(path)
}

/// Путь к файлу статистики или Err.
pub fn write_stats(
    base: &Path,
    format: &str,
    tasks: &[Task],
    filter: &TaskFilter,
    date_from: &str,
    date_to: &str,
) -> Result<PathBuf, String> {
    let st = statistics(tasks, filter);
    if st.count == 0 {
        return Err("Нет записей за выбранный период".to_string());
    }
    let dir = reports_dir(base);
    let ts = ts_stamp();
    let path = dir.join(format!("stats_{ts}.{format}"));

    match format {
        "txt" => {
            let body = stats_txt(&st, date_from, date_to);
            std::fs::write(&path, body).map_err(|e| e.to_string())?;
        }
        "md" => {
            let body = stats_md(&st, date_from, date_to);
            std::fs::write(&path, body).map_err(|e| e.to_string())?;
        }
        "csv" => {
            let mut body = String::new();
            body.push_str("\"СТАТИСТИКА\",\"Период: ");
            body.push_str(&fmt_period(date_from, date_to));
            body.push_str("\"\r\n");
            body.push_str(&row_csv(&[
                "Записей".to_string(),
                st.count.to_string(),
                "Общее время".to_string(),
                st.total_label.clone(),
            ]));
            body.push_str("\r\n");
            for (title, items) in stats_sections(&st) {
                if items.is_empty() {
                    continue;
                }
                body.push_str(&row_csv(&[title.to_string()]));
                body.push_str(&row_csv(&[
                    "Имя".to_string(),
                    "Задач".to_string(),
                    "Общее время".to_string(),
                    "Среднее".to_string(),
                ]));
                for item in items {
                    body.push_str(&row_csv(&[
                        item.name.clone(),
                        item.count.to_string(),
                        item.total_label.clone(),
                        item.avg_label.clone(),
                    ]));
                }
                body.push_str("\r\n");
            }
            write_cp1251(&path, &body)?;
        }
        "xlsx" => {
            write_stats_xlsx(&path, &st, date_from, date_to)?;
        }
        _ => return Err(format!("Неизвестный формат: {format}")),
    }
    Ok(path)
}

fn fmt_period(date_from: &str, date_to: &str) -> String {
    format!("{date_from} — {date_to}")
}

fn report_txt(list: &[&Task], secs: f64, tag_map: &HashMap<String, f64>, date_from: &str, date_to: &str, now: chrono::NaiveDateTime) -> String {
    let mut lines = Vec::new();
    lines.push("ОТЧЁТ ПО ВРЕМЕНИ".to_string());
    lines.push(format!("Период: {}", fmt_period(date_from, date_to)));
    lines.push("=".repeat(70));
    lines.push(String::new());
    lines.push(format!("Записей: {}", list.len()));
    lines.push(format!("Общее время: {}", fmt_td(secs)));
    lines.push(String::new());
    lines.push("ПО ТЕГАМ:".to_string());
    lines.push("-".repeat(40));
    let mut tags: Vec<(&String, &f64)> = tag_map.iter().collect();
    tags.sort_by(|a, b| a.0.cmp(b.0));
    for (name, s) in tags {
        lines.push(format!("  {:<30} {}", name, fmt_td(*s)));
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
            t.order,
            t.ranges_str(),
        ));
        if !t.comment.is_empty() {
            lines.push(format!("    Комментарий: {}", t.comment));
        }
    }
    lines.join("\n")
}

fn report_md(list: &[&Task], secs: f64, tag_map: &HashMap<String, f64>, date_from: &str, date_to: &str) -> String {
    let mut lines = Vec::new();
    lines.push(format!("# Отчёт по времени\n"));
    lines.push(format!("**Период:** {}\n", fmt_period(date_from, date_to)));
    lines.push(format!("**Записей:** {}  ", list.len()));
    lines.push(format!("**Общее время:** {}\n", fmt_td(secs)));
    lines.push("## По тегам\n".to_string());
    lines.push("| Тег | Время |".to_string());
    lines.push("|-----|-------|".to_string());
    let mut tags: Vec<(&String, &f64)> = tag_map.iter().collect();
    tags.sort_by(|a, b| a.0.cmp(b.0));
    for (name, s) in tags {
        lines.push(format!("| {} | {} |", name, fmt_td(*s)));
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
            t.order,
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

fn header_format() -> rust_xlsxwriter::Format {
    rust_xlsxwriter::Format::new()
        .set_bold()
        .set_font_color(rust_xlsxwriter::Color::White)
        .set_background_color(rust_xlsxwriter::Color::RGB(0x2196F3))
        .set_align(rust_xlsxwriter::FormatAlign::Center)
        .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter)
        .set_border(rust_xlsxwriter::FormatBorder::Thin)
}

fn write_report_xlsx(
    path: &Path,
    list: &[&Task],
    secs: f64,
    tag_map: &HashMap<String, f64>,
    date_from: &str,
    date_to: &str,
) -> Result<(), String> {
    use rust_xlsxwriter::Workbook;

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Отчёт").map_err(|e| e.to_string())?;

    let header = header_format();
    let mut row = 0u32;
    ws.write_string_with_format(row, 0, "ОТЧЁТ ПО ВРЕМЕНИ", &header).map_err(err)?;
    row += 1;
    ws.write_string(row, 0, &fmt_period(date_from, date_to)).map_err(err)?;
    row += 2;
    ws.write_string(row, 0, format!("Записей: {}", list.len())).map_err(err)?;
    ws.write_string(row, 1, format!("Общее время: {}", fmt_td(secs))).map_err(err)?;
    row += 2;
    ws.write_string(row, 0, "По тегам:").map_err(err)?;
    row += 1;
    let mut tags: Vec<(&String, &f64)> = tag_map.iter().collect();
    tags.sort_by(|a, b| a.0.cmp(b.0));
    for (name, s) in tags {
        ws.write_string(row, 0, name).map_err(err)?;
        ws.write_string(row, 1, fmt_td(*s)).map_err(err)?;
        row += 1;
    }
    row += 1;

    let headers = [
        "Дата начала", "Дата окончания", "Пользователь", "Номер заявки",
        "Клиент", "Тег", "Время", "Комментарий", "Диапазоны",
    ];
    for (col, h) in headers.iter().enumerate() {
        ws.write_string_with_format(row, col as u16, *h, &header).map_err(err)?;
    }
    row += 1;
    for t in list {
        let vals = [
            t.start_str(),
            t.end_str(),
            t.user.clone(),
            t.order.clone(),
            t.client.clone(),
            t.tags.join(", "),
            fmt_td(t.total_seconds(now_naive())),
            t.comment.clone(),
            t.ranges_str(),
        ];
        for (col, val) in vals.iter().enumerate() {
            ws.write_string(row, col as u16, val).map_err(err)?;
        }
        row += 1;
    }

    for col in 0..headers.len() {
        ws.set_column_width(col as u16, 18.0).map_err(err)?;
    }
    wb.save(path).map_err(err)
}

fn stats_sections(st: &Stats) -> [(&'static str, &[StatsLine]); 4] {
    [
        ("По пользователям", &st.by_user),
        ("По клиентам", &st.by_client),
        ("По тегам", &st.by_tag),
        ("По месяцам", &st.by_month),
    ]
}

fn stats_txt(st: &Stats, date_from: &str, date_to: &str) -> String {
    let mut lines = Vec::new();
    lines.push("СТАТИСТИКА".to_string());
    lines.push(format!("Период: {}", fmt_period(date_from, date_to)));
    lines.push("=".repeat(70));
    lines.push(String::new());
    lines.push(format!("Записей: {}", st.count));
    lines.push(format!("Общее время: {}", st.total_label));
    lines.push(format!("Среднее время задачи: {}", st.avg_label));
    lines.push(String::new());
    for (title, items) in stats_sections(st) {
        if items.is_empty() {
            continue;
        }
        lines.push(format!("{title}:"));
        lines.push("-".repeat(55));
        for item in items {
            lines.push(format!(
                "  {:<25} {:>6} задач {:>12} (ср. {})",
                item.name, item.count, item.total_label, item.avg_label
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
    lines.push(format!("**Общее время:** {}  ", st.total_label));
    lines.push(format!("**Среднее:** {}\n", st.avg_label));
    for (title, items) in stats_sections(st) {
        if items.is_empty() {
            continue;
        }
        lines.push(format!("## {title}\n"));
        lines.push("| Имя | Задач | Общее время | Среднее |".to_string());
        lines.push("|-----|-------|-------------|---------|".to_string());
        for item in items {
            lines.push(format!(
                "| {} | {} | {} | {} |",
                item.name, item.count, item.total_label, item.avg_label
            ));
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn write_stats_xlsx(path: &Path, st: &Stats, date_from: &str, date_to: &str) -> Result<(), String> {
    use rust_xlsxwriter::Workbook;

    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("Статистика").map_err(e2s)?;

    let header = header_format();
    let mut row = 0u32;
    ws.write_string_with_format(row, 0, "СТАТИСТИКА", &header).map_err(e2s)?;
    row += 1;
    ws.write_string(row, 0, &fmt_period(date_from, date_to)).map_err(e2s)?;
    row += 2;
    ws.write_string(row, 0, format!("Записей: {}", st.count)).map_err(e2s)?;
    ws.write_string(row, 1, format!("Общее время: {}", st.total_label)).map_err(e2s)?;
    ws.write_string(row, 2, format!("Среднее: {}", st.avg_label)).map_err(e2s)?;
    row += 2;

    for (title, items) in stats_sections(st) {
        if items.is_empty() {
            continue;
        }
        ws.write_string(row, 0, title).map_err(e2s)?;
        row += 1;
        for (col, h) in ["Имя", "Задач", "Общее время", "Среднее"].iter().enumerate() {
            ws.write_string_with_format(row, col as u16, *h, &header).map_err(e2s)?;
        }
        row += 1;
        for item in items {
            ws.write_string(row, 0, &item.name).map_err(e2s)?;
            ws.write_number(row, 1, item.count as f64).map_err(e2s)?;
            ws.write_string(row, 2, &item.total_label).map_err(e2s)?;
            ws.write_string(row, 3, &item.avg_label).map_err(e2s)?;
            row += 1;
        }
        row += 1;
    }
    for col in 0..4u16 {
        ws.set_column_width(col, 18.0).map_err(e2s)?;
    }
    wb.save(path).map_err(e2s)
}

fn err(e: rust_xlsxwriter::XlsxError) -> String {
    e.to_string()
}

fn e2s(e: rust_xlsxwriter::XlsxError) -> String {
    e.to_string()
}