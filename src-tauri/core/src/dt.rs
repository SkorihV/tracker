//! Форматы дат/времени, совместимые с Python-версией.
//!
//! Даты в файле хранятся как «дд.мм.гггг чч:мм» (минуты), время показывается
//! как ЧЧ:ММ:СС. В памяти храним полную точность (секунды), как и Python.

use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};

pub const DT_FMT: &str = "%d.%m.%Y %H:%M";
pub const DATE_KEY_FMT: &str = "%Y%m%d";
pub const MONTH_KEY_FMT: &str = "%Y%m";

/// Разбор «дд.мм.гггг чч:мм».
pub fn parse_dt(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    NaiveDateTime::parse_from_str(s, DT_FMT).ok()
}

/// Разбор даты «дд.мм.гггг» (для фильтров «с/по»).
pub fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%d.%m.%Y").ok()
}

/// Формат «дд.мм.гггг чч:мм».
pub fn fmt_dt(dt: NaiveDateTime) -> String {
    dt.format(DT_FMT).to_string()
}

pub fn now_naive() -> NaiveDateTime {
    Local::now().naive_local()
}

/// Ключ дня «ГГГГММДД» (для task_id и дневных/недельных бэкапов).
pub fn today_key() -> String {
    Local::now().format(DATE_KEY_FMT).to_string()
}

/// Понедельник текущей недели (ISO, неделя начинается с понедельника).
pub fn current_monday() -> NaiveDate {
    let today = Local::now().date_naive();
    today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64)
}

/// Формат длительности «ЧЧ:ММ:СС».
pub fn fmt_td(seconds: f64) -> String {
    let total = seconds.max(0.0) as i64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}